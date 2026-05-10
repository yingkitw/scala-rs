use crate::env::Environment;
use crate::value::Value;

pub fn register_stdlib(env: &mut Environment) {
    env.define("println", Value::BuiltinFunction { name: "__println__".into(), arity: 1 }, false);
    env.define("print", Value::BuiltinFunction { name: "__print__".into(), arity: 1 }, false);
    env.define("assert", Value::BuiltinFunction { name: "__assert__".into(), arity: 1 }, false);
    env.define("require", Value::BuiltinFunction { name: "__require__".into(), arity: 1 }, false);
    env.define("identity", Value::BuiltinFunction { name: "__identity__".into(), arity: 1 }, false);

    env.define("List", Value::BuiltinFunction { name: "__List_apply__".into(), arity: 0 }, false);
    env.define("Map", Value::BuiltinFunction { name: "__Map_apply__".into(), arity: 0 }, false);
    env.define("Some", Value::BuiltinFunction { name: "__Some__".into(), arity: 1 }, false);
    env.define("None", Value::String("None".into()), false);

    let math_obj = Value::Object {
        class_name: "math".into(),
        fields: {
            let mut f = std::collections::HashMap::new();
            f.insert("Pi".into(), Value::Double(std::f64::consts::PI));
            f.insert("E".into(), Value::Double(std::f64::consts::E));
            f
        },
        methods: {
            let mut m = std::collections::HashMap::new();
            m.insert("abs".into(), Value::BuiltinFunction { name: "__abs__".into(), arity: 1 });
            m.insert("max".into(), Value::BuiltinFunction { name: "__max__".into(), arity: 2 });
            m.insert("min".into(), Value::BuiltinFunction { name: "__min__".into(), arity: 2 });
            m.insert("pow".into(), Value::BuiltinFunction { name: "__pow__".into(), arity: 2 });
            m.insert("sqrt".into(), Value::BuiltinFunction { name: "__sqrt__".into(), arity: 1 });
            m
        },
    };
    env.define("math", math_obj, false);

    env.define("Range", Value::Object {
        class_name: "Range$".into(),
        fields: std::collections::HashMap::new(),
        methods: {
            let mut m = std::collections::HashMap::new();
            m.insert("apply".into(), Value::BuiltinFunction { name: "__range_to__".into(), arity: 2 });
            m
        },
    }, false);

    env.define("toInt", Value::BuiltinFunction { name: "__identity__".into(), arity: 1 }, false);
}
