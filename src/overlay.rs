use crate::Config;
use crate::Media;
use clap::Arg;
use gstreamer::{
    caps::Caps, event, message::MessageView, prelude::*, BusSyncReply, Element, ElementFactory,
    Pipeline, State,
};
use gstreamer_video::{prelude::VideoOverlayExtManual, VideoOverlay};
use num_rational::Ratio;
use std::{thread, time};
use x11rb::{
    connection::Connection,
    protocol::xproto::{
        ConnectionExt, // Trait
        CreateWindowAux,
        EventMask,
        WindowClass,
    },
    wrapper::ConnectionExt as ConnectionExtTrait,
};
pub fn create_arg<'a>() -> Arg<'a> {
    Arg::new("overlay")
        .long("overlay")
        .takes_value(true)
        .default_value("true")
        .required(false)
        .help("Show / Hide Overlay")
}

pub fn default() -> bool {
    true
}

pub const COMMAND_NAME: &str = "overlay";

pub fn show() {
    println!("Showing Overlay")
}
#[derive(Debug)]
pub struct CameraPreview {
    pub config: Config, // Not actually required here
    pub pipeline: Option<Pipeline>,
}

impl Media for CameraPreview {
    fn new(config: Config) -> Self {
        CameraPreview {
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
    fn create_pipeline(&mut self) {
        let rate = Ratio::new(self.config.framerate as i32, 1);
        const WIDTH: i32 = 400;
        // const WIDTH: i32 = 800;
        const HEIGHT: i32 = 300;
        // const HEIGHT: i32 = 600;
        const PADDING: i32 = 15;
        const FORMAT: &str = "YUV2";
        /* Window creation */
        let (conn, screen_num) = x11rb::connect(None).unwrap();
        let screen = &conn.setup().roots[screen_num];
        let win_id = conn.generate_id().unwrap();

        let screen_width = screen.width_in_pixels as u16;
        let screen_height = screen.height_in_pixels as u16;

        let window_width = WIDTH as u16;
        let window_height = HEIGHT as u16;

        let win_aux = CreateWindowAux::new()
            .event_mask(
                EventMask::EXPOSURE
                    | EventMask::STRUCTURE_NOTIFY
                    | EventMask::BUTTON1_MOTION
                    | EventMask::NO_EVENT,
            )
            .override_redirect(true as u32)
            .border_pixel(None)
            .background_pixel(screen.black_pixel);

        conn.create_window(
            screen.root_depth,
            win_id,
            screen.root,
            (screen_width - window_width - (PADDING as u16)) as i16,
            (screen_height - window_height - (PADDING as u16)) as i16,
            window_width,
            window_height,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &win_aux,
        )
        .unwrap();

        conn.map_window(win_id).unwrap();

        /* Gstreamer pipline message handler */
        let sync_handler_closure = move |_bus: &gstreamer::Bus, msg: &gstreamer::Message| {
            match msg.view() {
                MessageView::Element(element) => {
                    // handle the window
                    let video_overlay = element
                        .src()
                        .unwrap()
                        .dynamic_cast::<VideoOverlay>()
                        .unwrap();

                    conn.sync().unwrap();
                    unsafe {
                        video_overlay.set_window_handle(win_id as _);
                    }
                    BusSyncReply::Drop
                }

                _ => BusSyncReply::Pass,
            }
        };

        /* Pipeline creation */
        gstreamer::init().expect("cannot start gstreamer");
        let main_pipeline = Pipeline::new(Some("test-pipeline"));

        let source =
            ElementFactory::make("v4l2src", Some("source")).expect("Unable to make source");
        let sink = ElementFactory::make("xvimagesink", Some("sink")).expect("Unable to make sink");
        let rate_convert =
            ElementFactory::make("videorate", None).expect("Unable to make videorate");
        let video_convert =
            ElementFactory::make("videoconvert", None).expect("Unable to make videoconvert");
        let caps =
            ElementFactory::make("capsfilter", Some("filter")).expect("Unable to make capsfilter");

        let capsfilter = Caps::new_simple(
            "video/x-raw",
            &[
                ("format", &FORMAT),
                ("framerate", &(gstreamer::Fraction(rate))),
            ],
        );

        main_pipeline
            .add_many(&[&source, &caps, &rate_convert, &video_convert, &sink])
            .expect("Unable to add elements to pipeline");

        unsafe {
            source.set_data("num-buffers", 300);
            sink.set_data("sync", 0);
            caps.set_data("caps", capsfilter);

            match caps.data::<Caps>("caps") {
                Some(_data) => {
                    // println!("{:?}", data.as_ref());
                }
                None => println!("No data"),
            }
        };

        Element::link_many(&[&source, &caps, &rate_convert, &video_convert, &sink])
            .expect("Unable to link elements");

        let pipline_bus = main_pipeline.bus().expect("Unable to get pipeline bust");
        pipline_bus.set_sync_handler(sync_handler_closure);

        self.pipeline = Some(main_pipeline);
    }
}
