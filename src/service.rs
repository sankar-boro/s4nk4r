use async_std::task;
use futures::future::Ready; 
use std::future::Future;
use std::marker::PhantomData;

use crate::responder::Responder;
use crate::{FromRequest, scope::Scope};
use loony_service::{Service, ServiceFactory};
use crate::app::{BoxedRouteService};

pub trait Factory<Arg, Res, O>: Clone + 'static 
where 
  Res: Future<Output=O>, 
  O: Responder
{
  fn factory_call(&self, param: Arg) -> Res;
}

pub trait AppServiceFactory{
  fn register(&self);
}

pub struct ServiceConfig {
  pub routes:Vec<Scope>,
}

impl ServiceConfig {
  pub fn new() -> Self {
    ServiceConfig {
      routes: Vec::new(),
    }
  }
	
	pub fn service(&mut self, route: Scope) {
    self.routes.push(route);
  }
}

pub trait ServiceConfigFactory {
  fn get_routes(&self) -> &Vec<Scope>;
}

impl ServiceConfigFactory for ServiceConfig {
  fn get_routes(&self) -> &Vec<Scope> {
    &self.routes
  }
}

impl<T, A, Res, O> Factory<(A,), Res, O> for T 
where 
  T: Fn(A,) -> Res + Clone + 'static, 
  Res: Future<Output=O>,
  O: Responder,
{
  fn factory_call(&self, (one,): (A,)) -> Res {
    (self)(one)
  }
}

impl<T, A, B, Res, O> Factory<(A,B,), Res, O> for T 
where 
  T: Fn(A, B,) -> Res + Clone + 'static, 
  Res: Future<Output=O>,
  O: Responder,
{
  fn factory_call(&self, (one, two,): (A, B,)) -> Res {
    (self)(one, two)
  }
}

// Structs
pub struct Wrapper<T, Arg, Res, O> 
where 
  T: Factory<Arg, Res, O>,
  Res: Future<Output=O>,
  O: Responder,
{
  service: T,
  _t: PhantomData<(Arg, Res, O)>
}

pub struct Extract<Arg: FromRequest, S> {
  service: S,
  _t: PhantomData<Arg>
}

pub struct ExtractService<Arg: FromRequest, S> {
    service: S,
    _t: PhantomData<Arg>,
}

struct RouteServiceWrapper<T: Service> {
    service: T,
}

pub struct RouteNewService<T>
where
  T: ServiceFactory<
    Request=String
  >,
  T::Service: 'static,
{
  service: T,
}

/**
* Implementations
*
*/


// Struct Implementation

impl<T, Arg, Res, O> Clone for Wrapper<T, Arg, Res, O>
where
  T: Factory<Arg, Res, O>,
  Res: Future<Output=O>,
  O: Responder,
{
    fn clone(&self) -> Self {
      Self {
        service: self.service.clone(),
        _t: PhantomData,
      }
    }
}

impl<T, Arg, Res, O> Wrapper<T, Arg, Res, O> 
where 
  T: Factory<Arg, Res, O>,
  Res: Future<Output=O>,
  O: Responder,
  {
    // service: Fn(Arg) -> Res
    pub fn new(service: T) -> Self {
      Self {
        service,
        _t: PhantomData,
      }  
  }
}

impl<Arg: FromRequest, S> Extract<Arg, S> {
  pub fn new(service: S) -> Self {
    Self {
      service,
      _t: PhantomData,
    }
  }  
}

impl<Arg: FromRequest, S> AppServiceFactory for Extract<Arg, S> {
  fn register(&self) {
      
  }
}

// Trait Implementation

impl<T, Arg, Res, O> Service for Wrapper<T, Arg, Res, O> 
where 
  T: Factory<Arg, Res, O>,
  Res: Future<Output=O>,
  O: Responder,
{
  type Request = Arg;
  type Response = String;
  type Error = ();
  // type Future = Ready<Result<Self::Response, ()>>;
  
  fn call(&mut self, param: Self::Request) -> Self::Response {
    let t = self.service.factory_call(param);
    let r = task::block_on(t);
    r.respond()
  }
}

impl<Arg: FromRequest, S> Service for ExtractService<Arg, S>
where
    S: Service<
            Request = Arg,
            Response = String,
        > + Clone,
{
    type Request = String;
    type Response = String;
    type Error = ();

    fn call(&mut self, req: Self::Request) -> Self::Response {
      let t = Arg::from_request(req.clone());
      let b = self.service.call(t);
      b
    }
}


impl<Arg: FromRequest, S> ServiceFactory for Extract<Arg, S> 
where S: Service<
          Request = Arg,
          Response = String,
        > + Clone,
{
    type Request = String;
    type Response = String;
    type Service = ExtractService<Arg, S>;
    type Error = ();

    fn new_service(&self) -> Self::Service {
      ExtractService {
        service: self.service.clone(),
        _t: PhantomData,
      }
    }
}


impl<T> Service for RouteServiceWrapper<T>
where
    T: Service<
        Request = String,
        Response = String,
    >,
{
    type Request = String;
    type Response = String;
    type Error = ();

    fn call(&mut self, req: Self::Request) -> Self::Response {
      let a = &mut self.service;
      let b = a.call(req);
      b
    }
}

// impl Service for BoxedRouteService {
//     type Request = String;
//     type Response = String;
//     type Error = ();

//     fn call(&self, param: Self::Request) -> Self::Response {
//       (**self).call(param.clone())
//     }
// }

impl<T> RouteNewService<T>
where
  T: ServiceFactory<
    Request=String,
    Response=String,
  >,
  T::Service: 'static,
{
  pub fn new(service: T) -> Self {
    Self {
      service,
    }
  } 
}


impl<T> ServiceFactory for RouteNewService<T> 
where 
  T: ServiceFactory<
    Request=String,
    Response=String,
  >,
  T::Service: Service + 'static,
{
    type Request = String;

    type Response = String;

    type Service = BoxedRouteService;
    type Error = ();

    fn new_service(&self) -> Self::Service {
      let s = &self.service;
      let service = s.new_service();
      let d = Box::new(RouteServiceWrapper {
        service,
      });
      d
    }
}