use std::collections::HashMap;
use uuid::Uuid;
use crate::types::{Generator, Value, Variable};

pub fn generate_variables(input: HashMap<String, Variable>) -> HashMap<String, Value> {
    let mut variables = HashMap::new();
    for (var_name, var_value) in input {
        match var_value {
            Variable::Value(value) => variables.insert(var_name, value),
            Variable::Generator(generator) => match generator {
                Generator::Guid => {
                    variables.insert(var_name, Value::from(Uuid::new_v4().to_string()))
                }
            },
        };
    }
    return variables;
}
