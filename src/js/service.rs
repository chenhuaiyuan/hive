// use std::{
//     borrow::BorrowMut,
//     cell::RefCell,
//     convert::Infallible,
//     net::SocketAddr,
//     pin::Pin,
//     sync::Arc,
//     task::{Context, Poll},
// };

// use crate::{
//     error::{Error as WebError, Result},
//     js::js_request::{create_js_request, JsRequest},
// };
// use futures_util::Future;
// use http::{Request, Response};
// use hyper::{
//     server::conn::AddrStream,
//     service::{make_service_fn, service_fn, Service},
//     Body, Server,
// };

// use super::server::create_server;

// pub struct Svc(Arc<v8::Isolate>, SocketAddr, v8::Global<v8::String>);

// impl Service<Request<Body>> for Svc {
//     type Response = Response<Body>;
//     type Error = WebError;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

//     fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
//         Poll::Ready(Ok(()))
//     }

//     fn call(&mut self, req: Request<Body>) -> Self::Future {
//         let method = req.method().as_str().to_string();
//         let path = req.uri().path().to_string();
//         let js_req = JsRequest::new(req, self.1);
//         log::info!(
//             "Request -- remote address: {}, method: {}, uri: {}",
//             self.1,
//             method,
//             path
//         );
//         Box::pin(async move {
//             let scope = &mut v8::HandleScope::new(isolate);
//             let context = v8::Context::new(scope);
//             let scope = &mut v8::ContextScope::new(scope, context);

//             let function = create_server(scope);
//             let server_key = v8::String::new(scope, "server").unwrap();
//             let global = context.global(scope);
//             global.set(scope, server_key.into(), function.into());

//             let handle_str = v8::String::new(&mut scope, "serve").unwrap();
//             let handle = self.2.get(&mut scope, handle_str.into());
//             let recv = v8::undefined(*scope);
//             if let Some(handle) = handle {
//                 let handle = v8::Local::<v8::Function>::try_from(handle)?;
//                 let method = v8::String::new(&mut scope, &method).unwrap();
//                 let path = v8::String::new(&mut scope, &path).unwrap();
//                 let req = create_js_request(&mut scope, js_req).await?;
//                 let try_catch = v8::TryCatch::new(*scope);
//                 let resp = handle.call(
//                     &mut try_catch,
//                     recv.into(),
//                     &[method.into(), path.into(), req.into()],
//                 );
//                 if let Some(resp) = resp {
//                     if resp.is_true() {
//                         let data = true.to_string();
//                         Ok(Response::new(Body::from(data)))
//                     } else if resp.is_false() {
//                         let data = false.to_string();
//                         Ok(Response::new(Body::from(data)))
//                     } else if resp.is_string() {
//                         let data = resp.to_string(&mut scope);
//                         if let Some(data) = data {
//                             let data = data.to_rust_string_lossy(&mut scope);
//                             Ok(Response::new(Body::from(data)))
//                         } else {
//                             Ok(Response::new(Body::empty()))
//                         }
//                     } else {
//                         Ok(Response::new(Body::empty()))
//                     }
//                 } else {
//                     Ok(Response::new(Body::empty()))
//                 }
//             } else {
//                 Ok(Response::new(Body::empty()))
//             }
//         })
//     }
// }

// pub struct MakeSvc(pub Arc<v8::Isolate>, pub v8::Global<v8::String>);

// impl Service<&AddrStream> for MakeSvc {
//     type Response = Svc;
//     type Error = WebError;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

//     fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
//         Poll::Ready(Ok(()))
//     }

//     fn call(&mut self, stream: &AddrStream) -> Self::Future {
//         let remote_addr = stream.remote_addr();

//         Box::pin(async move { Ok(Svc(this.0, remote_addr, this.1)) })
//     }
// }

// async fn svc_handle<'s>(
//     req: Request<Body>,
//     isolate: Arc<v8::Isolate>,
//     remote_addr: SocketAddr,
//     handle_obj: v8::Local<'s, v8::Object>,
// ) -> Result<Response<Body>> {
//     let method = req.method().as_str().to_string();
//     let path = req.uri().path().to_string();
//     let js_req = JsRequest::new(req, remote_addr);
//     log::info!(
//         "Request -- remote address: {}, method: {}, uri: {}",
//         remote_addr,
//         method,
//         path
//     );
//     let mut isolate_handle = isolate.thread_safe_handle();
//     let handle_str = v8::String::new(&mut scope, "serve").unwrap();
//     let handle = handle_obj.get(&mut scope, handle_str.into());
//     let recv = v8::undefined(&mut *scope);
//     if let Some(handle) = handle {
//         let handle = v8::Local::<v8::Function>::try_from(handle)?;
//         let method = v8::String::new(&mut scope, &method).unwrap();
//         let path = v8::String::new(&mut scope, &path).unwrap();
//         let req = create_js_request(&mut scope, js_req).await?;
//         let mut try_catch = v8::TryCatch::new(*scope);
//         let resp = handle.call(
//             &mut try_catch,
//             recv.into(),
//             &[method.into(), path.into(), req.into()],
//         );
//         if let Some(resp) = resp {
//             if resp.is_true() {
//                 let data = true.to_string();
//                 Ok(Response::new(Body::from(data)))
//             } else if resp.is_false() {
//                 let data = false.to_string();
//                 Ok(Response::new(Body::from(data)))
//             } else if resp.is_string() {
//                 let data = resp.to_string(&mut try_catch);
//                 if let Some(data) = data {
//                     let data = data.to_rust_string_lossy(&mut try_catch);
//                     Ok(Response::new(Body::from(data)))
//                 } else {
//                     Ok(Response::new(Body::empty()))
//                 }
//             } else {
//                 Ok(Response::new(Body::empty()))
//             }
//         } else {
//             Ok(Response::new(Body::empty()))
//         }
//     } else {
//         Ok(Response::new(Body::empty()))
//     }
// }

// pub async fn js_service_run<'s>(
//     scope: Arc<&mut v8::HandleScope<'s>>,
//     handle_obj: v8::Local<'s, v8::Object>,
//     local_addr: SocketAddr,
// ) -> Result<()> {
//     let make_svc = make_service_fn(move |stream: &AddrStream| {
//         let remote_addr = stream.remote_addr();
//         async move {
//             Ok::<_, WebError>(service_fn(move |req| {
//                 svc_handle(req, scope.clone(), remote_addr, handle_obj)
//             }))
//         }
//     });
//     Server::bind(&local_addr).serve(make_svc).await?;
//     Ok(())
// }
