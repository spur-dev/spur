use crate::{api, Config, Media};
use futures::executor;
use gstreamer::{caps::Caps, event, prelude::*, Element, ElementFactory, Pipeline, State};
use num_rational::Ratio;
use std::{thread, time};

use tokio::runtime::Runtime; // TODO: Find a way to avoid this by spawning in tokio runtime
#[derive(Debug)]
pub struct Streamer {
    pub config: Config,
    pub pipeline: Option<Pipeline>,
}

impl Media for Streamer {
    fn new(config: Config) -> Self {
        Streamer {
            config,
            pipeline: None,
        }
    }

    fn start_pipeline(&mut self) {
        println!("starting stream pipeline"); // DEBUG

        match &self.pipeline {
            Some(pipeline) => pipeline
                .set_state(State::Playing)
                .expect("Could not start Streaming Pipeline"),
            None => panic!("Pipeline not created"),
        };
    }

    fn stop_stream(&self) {
        match &self.pipeline {
            Some(pipeline) => {
                pipeline.send_event(event::Eos::new());
                thread::sleep(time::Duration::from_secs(5)); // Hacky: but there should be a better way}
                pipeline
                    .set_state(State::Null)
                    .expect("Unable to set the pipeline to the `Null` state");

                // Getting preview url
                if let Some(vid) = &self.config.vid {
                    // TODO: avoid spawning runtimes because of the thread spawn
                    let rt = Runtime::new().unwrap();
                    let handle = rt.handle();
                    let preview_url = handle
                        .block_on(api::get_preview_url(vid))
                        .expect("Could not generate video id");

                    println!(
                        "You can see your recording here, once it is ready - {}",
                        preview_url
                    )
                }
            }
            None => panic!("Trying to stop pipeline before creating"),
        };
    }

    fn cancel_stream(&self) {
        println!("cancelling...");
        if let Some(pipeline) = &self.pipeline {
            pipeline.send_event(event::Eos::new());
            pipeline
                .set_state(State::Null)
                .expect("Unable to set the pipeline to the `Null` state");

            // sending cancel event to backend
            if let Some(vid) = &self.config.vid {
                // TODO: avoid spawning runtimes because of the thread spawn
                let rt = Runtime::new().unwrap();
                let handle = rt.handle();
                let _ = handle.block_on(api::cancel_recording(vid));
            }
        };
    }
    fn create_pipeline(&mut self) {
        // Asking backend for video id
        let vid = executor::block_on(api::get_new_video_id(&self.config.uid))
            .expect("Could not generate video id");
        self.config.vid = Some(vid);

        let rate = Ratio::new(self.config.framerate as i32, 1);

        // Pipeline creation
        gstreamer::init().expect("cannot start gstreamer");
        let main_pipeline = Pipeline::new(Some("recorder"));

        // Video elements
        let src_video = ElementFactory::make("ximagesrc", Some("desktop-video-source"))
            .expect("Unable to make desktop-video-source");
        let rate_video = ElementFactory::make("videorate", Some("desktop-video-framerate"))
            .expect("Unable to make desktop-video-framerate");
        let convert_video = ElementFactory::make("videoconvert", Some("desktop-video-converter"))
            .expect("Unable to make desktop-video-converter");
        let raw_video_caps = ElementFactory::make("capsfilter", Some("desktop-video-raw-caps"))
            .expect("Unable to make desktop-video-raw-caps");
        let encoder_video = ElementFactory::make("x264enc", Some("desktop-video-encoder"))
            .expect("Unable to make desktop-video-encoder");
        let encoder_video_caps =
            ElementFactory::make("capsfilter", Some("desktop-video-encoder-caps"))
                .expect("Unable to make desktop-video-encoder-caps");
        let queue_video = ElementFactory::make("queue2", Some("desktop-video-queue-1"))
            .expect("Unable to make desktop-video-queue-1");

        // Audio elements
        let src_audio = ElementFactory::make("pulsesrc", Some("desktop-audio-source"))
            .expect("Unable to make desktop-audio-source");
        let raw_audio_caps = ElementFactory::make("capsfilter", Some("desktop-raw-audio-caps"))
            .expect("Unable to make desktop-raw-audio-caps");
        let queue_audio = ElementFactory::make("queue2", Some("desktop-audio-queue"))
            .expect("Unable to make desktop-audio-queue");
        // let encoder_audio = ElementFactory::make("opusenc", Some("desktop-audio-encoder")).expect("Unable to make desktop-audio-encoder");

        // Mux and sink -- maybe sink, maybe rtmp
        let muxer =
            ElementFactory::make("flvmux", Some("mkv-muxer")).expect("Unable to make mkv-muxer"); // trying different muxer here
        let sink = ElementFactory::make("rtmpsink", Some(&self.config.get_target_path().as_str()))
            .expect("Unable to make mkv-filesink");

        // Adding video elements
        main_pipeline
            .add_many(&[
                &src_video,
                &rate_video,
                &convert_video,
                &raw_video_caps,
                &encoder_video,
                &encoder_video_caps,
                &queue_video,
            ])
            .expect("unable to add video elements to recording pipeline");
        // Adding audio elements
        // main_pipeline.add_many(&[&src_audio, &raw_audio_caps, &queue_audio, &encoder_audio]).expect("unable to add audio elements to recording pipeline");
        main_pipeline
            .add_many(&[&src_audio, &raw_audio_caps, &queue_audio])
            .expect("unable to add audio elements to recording pipeline");
        // Adding tail elements
        main_pipeline
            .add_many(&[&muxer, &sink])
            .expect("unable to add audio elements to recording pipeline");

        // Creating capsfilters
        let raw_video_capsfilter = Caps::builder("video/x-raw")
            .field("framerate", &(gstreamer::Fraction(rate)))
            .build();
        let encoded_video_capsfilter = Caps::builder("video/x-h264")
            .field("profile", &"constrained-baseline")
            .build();
        let raw_audio_capsfilter = Caps::builder("audio/x-raw")
            .field("framerate", &(gstreamer::Fraction::from(30)))
            .field("channels", 1)
            .field("rate", 44100) // does not work
            .build();
        // Setting properties
        src_video.set_property("use-damage", true).unwrap();

        raw_video_caps
            .set_property("caps", &raw_video_capsfilter)
            .unwrap();
        encoder_video_caps
            .set_property("caps", &encoded_video_capsfilter)
            .unwrap();
        raw_audio_caps
            .set_property("caps", &raw_audio_capsfilter)
            .unwrap();

        encoder_video
            .set_properties(&[
                (&"intra-refresh", &true),
                (&"vbv-buf-capacity", &(0 as u32)),
                (&"qp-min", &(30 as u32)),
                // (&"pass", &"pass1"),
                (&"key-int-max", &(36 as u32)),
                // (&"speed-preset", &"fast"),
                // (&"tune", &"stillimage"),
            ])
            .unwrap();
        queue_video
            .set_properties(&[
                (&"max-size-bytes", &(0 as u32)),
                (&"max-size-buffers", &(0 as u32)),
                // (&"max-size-time", &(0 as u32)),
            ])
            .unwrap();
        queue_video.set_property("max-size-time", 0 as u64).unwrap();

        // encoder_audio.set_property("bitrate-type", "constrained-vbr").unwrap();
        queue_audio
            .set_properties(&[
                (&"max-size-bytes", &(0 as u32)),
                (&"max-size-buffers", &(0 as u32)),
                // (&"max-size-time", &(0 as u32)),
            ])
            .unwrap();
        queue_audio.set_property("max-size-time", 0 as u64).unwrap();
        sink.set_property("location", &self.config.get_target_path())
            .unwrap();

        // Linking video elements
        Element::link_many(&[
            &src_video,
            &rate_video,
            &convert_video,
            &raw_video_caps,
            &encoder_video,
            &encoder_video_caps,
            &queue_video,
        ])
        .expect("unable to link video elements in recording pipeline");
        // Linking audio elements
        // Element::link_many(&[&src_audio, &raw_audio_caps, &queue_audio, &encoder_audio]).expect("unable to link audio elements in recording pipeline");
        Element::link_many(&[&src_audio, &raw_audio_caps, &queue_audio])
            .expect("unable to link audio elements in recording pipeline");
        // Linking tail elements
        Element::link_many(&[&muxer, &sink])
            .expect("unable to link audio elements in recording pipeline");

        // let video_pipeline_source_pad = queue_video.request_pad_simple("src").expect("unable to get video_pipeline_source_pad");
        // let audio_pipeline_source_pad = encoder_audio.request_pad_simple("src").expect("unable to get audio_pipeline_source_pad");

        let video_pipeline_source_pad = &queue_video.src_pads()[0];
        // let audio_pipeline_source_pad = &encoder_audio.src_pads()[0];
        let audio_pipeline_source_pad = &queue_audio.src_pads()[0];

        let muxer_video_sink_pad = muxer
            .compatible_pad(video_pipeline_source_pad, Some(&encoded_video_capsfilter))
            .expect("Unable to get muxer_video_sink_pad");
        let muxer_audio_sink_pad = muxer
            .compatible_pad(audio_pipeline_source_pad, Some(&raw_audio_capsfilter))
            .expect("Unable to get muxer_audio_sink_pad");

        queue_video
            .link_pads(
                Some(video_pipeline_source_pad.name().as_str()),
                &muxer,
                Some(muxer_video_sink_pad.name().as_str()),
            )
            .unwrap(); // Video to muxer // TODO (probably overcomplicating): use `link_pad` with sync handler

        // encoder_audio.link_pads(Some(audio_pipeline_source_pad.name().as_str()), &muxer, Some(muxer_audio_sink_pad.name().as_str())).unwrap(); // Audio to muxer // TODO (probably overcomplicating): use `link_pad` with sync handler
        queue_audio
            .link_pads(
                Some(audio_pipeline_source_pad.name().as_str()),
                &muxer,
                Some(muxer_audio_sink_pad.name().as_str()),
            )
            .unwrap(); // Audio to muxer // TODO (probably overcomplicating): use `link_pad` with sync handler

        self.pipeline = Some(main_pipeline)
    }
}
