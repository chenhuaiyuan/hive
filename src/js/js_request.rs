use crate::error::Result;
use crate::request::{HttpData, Request};
use hyper::{Body, Request as HyperRequest};
use std::cell::RefCell;
use std::net::SocketAddr;
use std::rc::Rc;

pub struct JsRequest(Request);

impl JsRequest {
    pub fn new(req: HyperRequest<Body>, remote_addr: SocketAddr) -> Self {
        Self(Request { req, remote_addr })
    }
}

fn generate_Object<'s>(
    scope: &mut v8::HandleScope<'s>,
    tab: v8::Local<'s, v8::Object>,
    mut cap: Vec<String>,
    val: String,
) -> Result<v8::Local<'s, v8::Object>> {
    if cap.is_empty() {
        return Ok(tab);
    }
    let index = cap.remove(0);
    let len = cap.len();
    let num = index.parse::<u32>();
    let index = v8::String::new(scope, &index).unwrap();
    if let Ok(i) = num {
        // let i = v8::Integer::new(scope, idx);
        if len == 0 {
            let val = v8::String::new(scope, &val).unwrap();
            tab.set_index(scope, i, val.into());
        } else {
            let table = tab.get_index(scope, i);
            if let Some(t) = table {
                let t = t.to_object(scope);
                if let Some(t) = t {
                    let temp = generate_Object(scope, t, cap, val)?;
                    tab.set_index(scope, i, temp.into());
                }
            } else {
                let temp_tab = v8::Object::new(scope);
                let t = generate_Object(scope, temp_tab, cap, val)?;
                tab.set_index(scope, i, t.into());
            }
        }
        return Ok(tab);
    } else if len == 0 {
        let val = v8::String::new(scope, &val).unwrap();
        tab.set(scope, index.into(), val.into());
        return Ok(tab);
    } else {
        let table = tab.get(scope, index.into());
        if let Some(t) = table {
            let t = t.to_object(scope);
            if let Some(t) = t {
                let temp = generate_Object(scope, t, cap, val)?;
                tab.set(scope, index.into(), temp.into());
            }
        } else {
            let temp_tab = v8::Object::new(scope);
            let t = generate_Object(scope, temp_tab, cap, val)?;
            tab.set(scope, index.into(), t.into());
        }
        return Ok(tab);
    }
}

impl JsRequest {
    pub async fn create_params<'s>(
        self,
        scope: &mut v8::HandleScope<'s>,
    ) -> Result<v8::Local<'s, v8::Function>> {
        let scope_1 = Rc::new(RefCell::new(scope));
        let scope_2 = Rc::clone(&scope_1);
        let scope_3 = Rc::clone(&scope_1);
        let f1 = |mut param: HttpData<v8::Local<'s, v8::Value>>,
                  param_key: String,
                  fields: Vec<String>,
                  val: String| {
            let param_value = param.get(&param_key);
            let mut scope_1 = scope_1.as_ref().borrow_mut();
            if let Some(value) = param_value {
                if let Some(value) = value.to_object(&mut scope_1) {
                    let temp_table = generate_Object(&mut scope_1, value, fields, val)?;
                    param.insert(param_key, temp_table.into());
                }
            } else {
                let temp = v8::Object::new(&mut scope_1);
                let temp_table = generate_Object(&mut scope_1, temp, fields, val)?;
                param.insert(param_key, temp_table.into());
            }
            Ok(param)
        };
        let f2 = |mut param: HttpData<v8::Local<'s, v8::Value>>, key: String, val: String| {
            let mut scope_2 = scope_2.as_ref().borrow_mut();
            let val = v8::String::new(&mut scope_2, &val);
            if let Some(val) = val {
                param.insert(key, val.into());
            }
            Ok(param)
        };
        let params = self.0.params(f1, f2).await?;
        let mut scope_3 = scope_3.as_ref().borrow_mut();
        let temp_object = v8::Object::new(&mut scope_3);
        for (key, val) in params {
            let key = v8::String::new(&mut scope_3, &key);
            if let Some(key) = key {
                temp_object.set(&mut scope_3, key.into(), val);
            }
        }
        let func = v8::FunctionTemplate::new(
            &mut scope_3,
            |_: &mut v8::HandleScope, _: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue| {
                rv.set(temp_object.into());
            },
        )
        .get_function(&mut scope_3)
        .unwrap();

        Ok(func)
    }
}

pub async fn create_js_request<'s>(
    scope: &mut v8::HandleScope<'s>,
    js_req: JsRequest,
) -> Result<v8::Local<'s, v8::Object>> {
    let js_request = v8::ObjectTemplate::new(scope);
    js_request.set(
        v8::String::new(scope, "params").unwrap().into(),
        js_req.create_params(scope).await?.into(),
    );
    let obj = js_request.new_instance(scope).unwrap();
    Ok(obj)
}
