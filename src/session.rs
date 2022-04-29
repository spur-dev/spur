use crate::{
    options::SType, overlay::CameraPreview, recorder::Recorder, streamer::Streamer, Config, Media,
};
use std::sync::{mpsc, Arc, Mutex};
#[derive(Debug)]
pub enum Task {
    Overlay(CameraPreview),
    Record(Recorder),
    Stream(Streamer),
}

#[derive(Debug)]
pub struct Session {
    pub s_type: SType,
    pub overlay: bool,
    pub reciever: Arc<Mutex<mpsc::Receiver<()>>>,
    pub sender: mpsc::Sender<()>,
    pub pipeline_channels: Vec<mpsc::Sender<()>>,
    pub config: Config,
    pub tasks: Vec<Task>,
}

impl Session {
    pub fn new(config: Config) -> Self {
        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        Session {
            s_type: config.s_type,
            reciever: rx,
            sender: tx,
            pipeline_channels: vec![],
            overlay: config.overlay,
            config,
            tasks: vec![],
        }
    }

    pub fn start_media_pipeline(&self) -> Task {
        let conf = self.config.clone();
        if self.s_type == SType::Record {
            let mut task_obj = Recorder::new(conf.clone());
            task_obj.create_pipeline();
            task_obj.start_pipeline();
            return Task::Record(task_obj);
        } else {
            println!("Stream does not currently work");
            let mut task_obj = Streamer::new(conf.clone());
            task_obj.create_pipeline();
            task_obj.start_pipeline();
            return Task::Stream(task_obj);
        }
    }

    pub fn start_overlay_pipeline(&self) -> Task {
        let mut task_obj = CameraPreview::new(self.config.clone());
        task_obj.create_pipeline();
        task_obj.start_pipeline();
        return Task::Overlay(task_obj);
    }
    pub fn start(&mut self) {
        println!("{:?}", self); // DEBUG
        self.tasks.push(self.start_media_pipeline());
        if self.overlay {
            self.tasks.push(self.start_overlay_pipeline());
        }
    }

    pub fn end(&mut self) {
        while let Some(task) = self.tasks.pop() {
            match task {
                Task::Record(obj) => obj.stop_stream(),
                Task::Stream(obj) => obj.stop_stream(),
                Task::Overlay(obj) => obj.stop_stream(),
            };
        }
    }

    pub fn cancel(&mut self) {
        while let Some(task) = self.tasks.pop() {
            match task {
                Task::Record(obj) => obj.cancel_stream(),
                Task::Stream(obj) => obj.cancel_stream(),
                Task::Overlay(obj) => obj.cancel_stream(),
            };
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Session::new(Config::default())
    }
}
