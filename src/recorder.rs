use crate::Config;
use crate::Media;
use gstreamer::{caps::Caps, event, prelude::*, Element, ElementFactory, Pipeline, State};
use num_rational::Ratio;
use std::thread;
use std::time;
#[derive(Debug)]
pub struct Recorder {
    pub config: Config,
    pub pipeline: Option<Pipeline>,
}

impl Media for Recorder {
    fn new(config: Config) -> Self {
        Recorder {
            config,
            pipeline: None,
        }
    }

    fn start_pipeline(&mut self) {
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

                println!("Got it! Closing...");
            }
            None => panic!("Pipeline not created"),
        };
    }

    fn cancel_stream(&self) {}
    // fn handle_stream(&self) {
    fn create_pipeline(&mut self) {
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
        let src_audio = ElementFactory::make("alsasrc", Some("desktop-audio-source"))
            .expect("Unable to make desktop-audio-source");
        let raw_audio_caps = ElementFactory::make("capsfilter", Some("desktop-raw-audio-caps"))
            .expect("Unable to make desktop-raw-audio-caps");
        let queue_audio = ElementFactory::make("queue2", Some("desktop-audio-queue"))
            .expect("Unable to make desktop-audio-queue");
        let encoder_audio = ElementFactory::make("voaacenc", Some("desktop-audio-encoder"))
            .expect("Unable to make desktop-audio-encoder");

        // Mux and sink -- maybe sink, maybe rtmp
        let muxer = ElementFactory::make("matroskamux", Some("mkv-muxer"))
            .expect("Unable to make mkv-muxer"); // trying different muxer here
        let sink = ElementFactory::make("filesink", Some("mkv-filesink"))
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
        main_pipeline
            .add_many(&[&src_audio, &raw_audio_caps, &queue_audio, &encoder_audio])
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
            .field("framerate", &(gstreamer::Fraction(rate)))
            .field("channels", 1)
            .field("rate", 48000) // does not work
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
                (&"key-int-max", &(36 as u32)),
                // (&"pass", &"pass1"),
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
        sink.set_property("location", &self.config.path).unwrap();

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
        Element::link_many(&[&src_audio, &raw_audio_caps, &queue_audio, &encoder_audio])
            .expect("unable to link audio elements in recording pipeline");
        // Linking tail elements
        queue_video.link(&muxer).unwrap(); // Video to muxer // TODO (probably overcomplicating): use `link_pad` with sync handler
        encoder_audio.link(&muxer).unwrap(); // Audio to muxer // TODO (probably overcomplicating): use `link_pad` with sync handler
        Element::link_many(&[&muxer, &sink])
            .expect("unable to link audio elements in recording pipeline");

        self.pipeline = Some(main_pipeline);
    }
}
