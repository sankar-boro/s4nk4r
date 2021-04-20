use std::{
    net::TcpStream, 
    sync::mpsc::Receiver
};
use crate::App;
use crate::builder::Builder;

pub type AppInstance = Box<dyn Fn() -> App + 'static>;

pub struct HttpServer {
    app: AppInstance,
    builder: Builder,
}

impl HttpServer {
    pub fn new<F: Fn() -> App + 'static>(app: F) -> Self {
        Self { 
            app: Box::new(app), 
            builder: Builder::new(),
        }
    }

    fn start(&mut self) {
        let app = (self.app)();
        let services = &app.config;
        let scopes = services.get_services();

        for scope in scopes.iter() {
            for route in scope.services.iter() {
                let s = route.new_service();
                println!("{}", s.path);
            }
        }
    }

    pub fn run(&mut self) {
        self.start();
    }

    fn accept(&self, _: Receiver<TcpStream>) {
    }
}