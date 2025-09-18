use nu_plugin::{JsonSerializer, Plugin, PluginCommand, serve_plugin};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Span, Type, Value,
};

mod parser;
use parser::{parse_posix_exports, exports_to_nushell};

struct FromPosixPlugin;

impl Plugin for FromPosixPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(FromPosix)]
    }
}

struct FromPosix;

impl PluginCommand for FromPosix {
    type Plugin = FromPosixPlugin;

    fn name(&self) -> &str {
        "from posix"
    }

    fn signature(&self) -> nu_protocol::Signature {
        nu_protocol::Signature::build("from posix")
            .input_output_types(vec![
                (Type::String, Type::String),
            ])
            .category(Category::Formats)
    }

    fn description(&self) -> &str {
        "Convert POSIX export statements to Nushell $env assignments"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: r#"'export FOO=bar' | from posix"#,
                description: "Convert a single export statement",
                result: Some(Value::string("$env.FOO = bar", Span::unknown())),
            },
            Example {
                example: r#"'export FOO=bar && export BAZ=qux' | from posix"#,
                description: "Convert multiple exports on the same line",
                result: Some(Value::string("$env.FOO = bar\n$env.BAZ = qux", Span::unknown())),
            },
            Example {
                example: r#"'export PATH="/usr/bin:/bin"' | from posix"#,
                description: "Convert export with quoted value",
                result: Some(Value::string(r#"$env.PATH = "/usr/bin:/bin""#, Span::unknown())),
            },
        ]
    }

    fn run(
        &self,
        _plugin: &FromPosixPlugin,
        _engine: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let span = call.head;

        // get input as string
        let input_string = match input {
            PipelineData::Value(Value::String { val, .. }, _) => val,
            PipelineData::ListStream(stream, _) => {
                let values: Vec<Value> = stream.into_iter().collect();
                if values.len() == 1 {
                    if let Value::String { val, .. } = &values[0] {
                        val.clone()
                    } else {
                        return Err(LabeledError::new("Input must be a string")
                            .with_label("expected string input", span));
                    }
                } else {
                    // join multiple string values with newlines
                    values.into_iter()
                        .filter_map(|v| match v {
                            Value::String { val, .. } => Some(val),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            PipelineData::Value(Value::List { vals, .. }, _) => {
                vals.into_iter()
                    .filter_map(|v| match v {
                        Value::String { val, .. } => Some(val),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            _ => {
                return Err(LabeledError::new("Input must be a string")
                    .with_label("expected string input", span));
            }
        };

        // parse POSIX exports
        let exports = parse_posix_exports(&input_string);

        // convert to Nushell format
        let nushell_output = exports_to_nushell(exports);

        // return as string value
        Ok(PipelineData::Value(
            Value::string(nushell_output, span),
            None,
        ))
    }
}

fn main() {
    serve_plugin(&FromPosixPlugin, JsonSerializer {})
}