**clean_cli** is in programm command line parser using for repl.

## Example

```rust
let cli = Cli::<()>::builder()
        .command(CommandBuilder::with_name("cmd")
            .use_value(ArgType::Bool)
            .parameter(Parameter::with_name("bool")
                .value_type(ArgType::Bool)
                .alias("b")
                .alias("bb")
            )
            .parameter(Parameter::with_name("int")
                .value_type(ArgType::Int)
                .alias("i")
                .alias("ii")
            )
            .parameter(Parameter::with_name("float")
                .value_type(ArgType::Float)
                .alias("f")
                .alias("ff")
            )
            .parameter(Parameter::with_name("string")
                .value_type(ArgType::String)
                .alias("s")
                .alias("ss")
            )
            .handler(|ctx| {
                let mut expect_params = HashSet::new();
                expect_params.insert("bool");
                expect_params.insert("int");
                expect_params.insert("float");
                expect_params.insert("string");
                expect_params.insert("value");

                if let Some(arg) = &ctx.units.last().unwrap().value() {
                    if let ArgValue::Bool(v) = arg {
                        assert!(!*v);
                        expect_params.remove("value");
                    }
                } 

                if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("bool") {
                    assert_eq!("bool", param.name.as_str());
                    if let ArgValue::Bool(v) = arg {
                        assert!(*v);
                        expect_params.remove("bool");
                    }
                } 

                if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("int") {
                    assert_eq!("int", param.name.as_str());
                    if let ArgValue::Int(v) = arg {
                        assert_eq!(*v, 42_i64);
                        expect_params.remove("int");
                    }
                }
                
                if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("float") {
                    assert_eq!("float", param.name.as_str());
                    if let ArgValue::Float(v) = arg {
                        assert_eq!(*v, 4.2_f64);
                        expect_params.remove("float");
                    }
                }
                
                if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("string") {
                    assert_eq!("string", param.name.as_str());
                    if let ArgValue::String(v) = arg {
                        assert_eq!(*v, "bla");
                        expect_params.remove("string");
                    }
                }
                
                if !expect_params.is_empty() {
                    panic!("parameters not found: {:?}", expect_params)
                };
            })
        )
        .build();

    match cli.exec_line("cmd false --bool --int 42 --float 4.2 --string bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err)
    }
```

Send all questions and wishes to <bugrazoid@yandex.ru>