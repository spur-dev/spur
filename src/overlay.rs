use crate::{Config, Media};
use clap::Arg;
use num_rational::Ratio;
use std::{sync::Arc, thread, time};

use gstreamer::{
    caps::Caps, event, message::MessageView, prelude::*, BusSyncReply, Element, ElementFactory,
    Pipeline, State,
};
use gstreamer_video::{prelude::VideoOverlayExtManual, VideoOverlay};

use x11rb::{
    connection::Connection,
    protocol::{
        xproto::{
            ConfigureWindowAux,
            ConnectionExt, // Trait
            CreateWindowAux,
            EventMask,
            WindowClass,
        },
        Event,
    },
    wrapper::ConnectionExt as ConnectionExtTrait,
};

/* Utils */
struct Dimension2D<T> {
    width: T,
    height: T,
}

struct Coordinate2D<T> {
    x: T,
    y: T,
}

impl<T> Dimension2D<T> {
    fn new(width: T, height: T) -> Self {
        Dimension2D::<T> { width, height }
    }
}

impl<T> Coordinate2D<T> {
    fn new(x: T, y: T) -> Self {
        Coordinate2D::<T> { x, y }
    }
}

#[allow(dead_code)] // todo: implement cli option for default overlay position
enum InitialPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Default for InitialPosition {
    fn default() -> Self {
        InitialPosition::BottomRight
    }
}

fn coordinates_for_initial_overlay(
    screen: &Dimension2D<u16>,
    window: &Dimension2D<u16>,
    padding: &Coordinate2D<u16>,
    position: InitialPosition,
) -> Coordinate2D<i16> {
    match position {
        InitialPosition::TopLeft => Coordinate2D::<i16>::new(
            (screen.width + padding.x) as i16,
            (screen.height + padding.y) as i16,
        ),
        InitialPosition::TopRight => Coordinate2D::<i16>::new(
            (screen.width - window.width - padding.x) as i16,
            (screen.height + padding.y) as i16,
        ),
        InitialPosition::BottomLeft => Coordinate2D::<i16>::new(
            (screen.width + padding.x) as i16,
            (screen.height - window.height - padding.y) as i16,
        ),
        InitialPosition::BottomRight => Coordinate2D::<i16>::new(
            (screen.width - window.width - padding.x) as i16,
            (screen.height - window.height - padding.y) as i16,
        ),
    }
}

pub fn create_arg<'a>() -> Arg<'a> {
    Arg::new("overlay")
        .long("overlay")
        .takes_value(true)
        .default_value("true")
        .required(false)
        .help("Show / Hide Overlay")
}

pub fn default() -> bool {
    // todo: convert to default trait implementation
    // Could seperate overlay into overlay_pipeline and overlay_option
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
                pipeline.send_event(event::Eos::new()); // todo: add handler
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
        // let window_dimensions = Dimension2D::<u16>::new(800, 600);
        let window_dimensions = Dimension2D::<u16>::new(400, 300);
        let padding = Coordinate2D::<u16>::new(15, 15);
        const FORMAT: &str = "YUV2";
        let rate = Ratio::new(self.config.framerate as i32, 1);

        /* Window creation */
        let (conn, screen_num) = x11rb::connect(None).unwrap();
        let conn = Arc::new(conn); // will be shared with gstreamer xvimagesink
        let screen = &conn.setup().roots[screen_num];
        let win_id = conn.generate_id().unwrap();

        let screen_dimensions =
            Dimension2D::<u16>::new(screen.width_in_pixels, screen.height_in_pixels);

        let win_aux = CreateWindowAux::new()
            .event_mask(
                EventMask::BUTTON1_MOTION, // EventMask::STRUCTURE_NOTIFY // todo: implement resizing
            )
            .override_redirect(true as u32)
            .border_pixel(None)
            .background_pixel(screen.black_pixel);

        let window_coordinates = coordinates_for_initial_overlay(
            &screen_dimensions,
            &window_dimensions,
            &padding,
            InitialPosition::default(),
        );
        conn.create_window(
            screen.root_depth,
            win_id,
            screen.root,
            window_coordinates.x,
            window_coordinates.y,
            window_dimensions.width,
            window_dimensions.height,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &win_aux,
        )
        .unwrap();

        conn.map_window(win_id).unwrap();

        /* Gstreamer pipline message handler */
        let conn1 = conn.clone();
        let win_id1 = win_id; // todo: figure out how to specify move, and referrence to variables in closures
        let sync_handler_closure = move |_bus: &gstreamer::Bus, msg: &gstreamer::Message| {
            match msg.view() {
                MessageView::Element(element) => {
                    // handle the window
                    let video_overlay = element
                        .src()
                        .unwrap()
                        .dynamic_cast::<VideoOverlay>()
                        .unwrap();

                    let _ = &conn1.sync().unwrap();
                    unsafe {
                        video_overlay.set_window_handle(win_id1 as _);
                    }
                    BusSyncReply::Drop
                }

                _ => BusSyncReply::Pass,
            }
        };

        /* X11 window event handler */
        // todo: add cleanup for join handle
        thread::spawn(move || {
            loop {
                let original_pointer_position =
                    &conn.query_pointer(win_id).unwrap().reply().unwrap();

                let new_window_coordinates = Coordinate2D::<i16>::new(
                    original_pointer_position.root_x - original_pointer_position.win_x,
                    original_pointer_position.root_y - original_pointer_position.win_y,
                );
                if let Ok(event) = &conn.wait_for_event() {
                    match event {
                        /* Dragging Window */
                        Event::MotionNotify(motion_event) => {
                            let deltax = original_pointer_position.win_x - motion_event.event_x;
                            let deltay = original_pointer_position.win_y - motion_event.event_y;

                            // Hacky, but works
                            if deltax.abs() < 75 && deltay.abs() < 75 {
                                let mut new_attributes = ConfigureWindowAux::new();
                                /* ConfigureWindowAux methods .x() and .y() do not work */
                                new_attributes.x = Some((new_window_coordinates.x - deltax) as i32);
                                new_attributes.y = Some((new_window_coordinates.y - deltay) as i32);

                                let _ = &conn.configure_window(win_id, &new_attributes).unwrap();

                                conn.map_window(win_id).unwrap();
                            }
                        }

                        _ => println!("Unwanted event recieved, please report this issue"),
                    }
                };
                conn.flush().unwrap();
            }
        });

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
