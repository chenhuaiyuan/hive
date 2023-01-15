use v8::{FunctionCallbackArguments, HandleScope, Local, ReturnValue};

pub fn create_server<'s>(scope: &mut HandleScope<'s>) -> Local<'s, v8::Function> {
    v8::FunctionTemplate::new(
        scope,
        |scope: &mut HandleScope, _: FunctionCallbackArguments, mut rv: ReturnValue| {
            let code = v8::String::new(scope, include_str!("./server.js")).unwrap();
            let script = v8::Script::compile(scope, code, None).unwrap();
            let data = script.run(scope).unwrap();
            rv.set(data);
        },
    )
    .get_function(scope)
    .unwrap()
}
