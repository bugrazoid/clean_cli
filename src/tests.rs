use crate::{
    traits::{Config, DefaultHelpFormatter, Printer},
    ArgType, ArgValue, Cli, CommandBuilder, Parameter,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    fmt::Debug,
    marker::PhantomData,
    rc::Rc,
};

pub struct Test<R>(PhantomData<R>);
impl<R: Default + Debug + 'static> Config for Test<R> {
    type Result = R;
    type HelpFormatter = DefaultHelpFormatter;
    type PrinterInput = String;
    type Printer = TestPrinter<Self>;
}
impl<R> Default for Test<R> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Default)]
pub struct TestPrinter<T: Config>(pub Rc<RefCell<T::PrinterInput>>);
impl<T: Config> Printer<T> for TestPrinter<T>
where
    T::PrinterInput: std::fmt::Display,
{
    fn print(&self, input: T::PrinterInput) {
        println!("{}", &input);
        *(*self.0).borrow_mut() = input;
    }
}

fn some_fn(ctx: crate::context::Context<Test<()>>) {
    if let Some(unit) = ctx.command_units().last() {
        assert_eq!(unit.command.0, "cmd");
    } else {
        panic!("context units empty");
    }
}

#[test]
fn use_regular_function() {
    let _cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(Parameter::with_name("param"))
                .handler(some_fn),
        )
        .build();
}

#[test]
fn command() {
    let is_triggered = Rc::new(Cell::new(false));
    let is_triggered_closure = is_triggered.clone();
    let cli = <Cli<Test<()>>>::builder()
        .command(CommandBuilder::with_name("cmd").handler(move |_| {
            is_triggered_closure.set(true);
        }))
        .build();

    match cli.exec_line("cmd") {
        Ok(_) => {}
        Err(e) => panic!("{:?}", e),
    }

    assert!(is_triggered.get());
}

#[test]
fn command_with_bool_param() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("flag")
                        .value_type(ArgType::Bool)
                        .alias("f")
                        .alias("fl"),
                )
                .handler(|ctx| {
                    if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("flag") {
                        assert_eq!("flag", param.name.as_str());
                        if let ArgValue::Bool(v) = arg {
                            assert!(*v)
                        }
                    } else {
                        panic!("parameter not found")
                    };
                }),
        )
        .build();

    match cli.exec_line("cmd --flag") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd --fl") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -f") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -f") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_two_bool_param() {
    let flags = Rc::new(Cell::new((false, false)));
    let flags_move = flags.clone();

    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("flag1")
                        .value_type(ArgType::Bool)
                        .alias("1"),
                )
                .parameter(
                    Parameter::with_name("flag2")
                        .value_type(ArgType::Bool)
                        .alias("2"),
                )
                .handler(move |ctx| {
                    if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("flag1") {
                        assert_eq!("flag1", param.name.as_str());
                        if let ArgValue::Bool(v) = arg {
                            assert!(*v);
                            let mut tmp = flags_move.get();
                            tmp.0 = true;
                            flags_move.set(tmp);
                        }
                    };

                    if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("flag2") {
                        assert_eq!("flag2", param.name.as_str());
                        if let ArgValue::Bool(v) = arg {
                            assert!(*v);
                            let mut tmp = flags_move.get();
                            tmp.1 = true;
                            flags_move.set(tmp);
                        }
                    }
                }),
        )
        .build();

    match cli.exec_line("cmd --flag1") {
        Ok(_) => {
            assert!(flags.get().0);
            assert!(!flags.get().1);
        }
        Err(err) => panic!("{:?}", err),
    }
    flags.set((false, false));

    match cli.exec_line("cmd -1") {
        Ok(_) => {
            assert!(flags.get().0);
            assert!(!flags.get().1);
        }
        Err(err) => panic!("{:?}", err),
    }
    flags.set((false, false));

    match cli.exec_line("cmd --flag2") {
        Ok(_) => {
            assert!(!flags.get().0);
            assert!(flags.get().1);
        }
        Err(err) => panic!("{:?}", err),
    }
    flags.set((false, false));

    match cli.exec_line("cmd -2") {
        Ok(_) => {
            assert!(!flags.get().0);
            assert!(flags.get().1);
        }
        Err(err) => panic!("{:?}", err),
    }
    flags.set((false, false));

    match cli.exec_line("cmd --flag1 --flag2") {
        Ok(_) => {
            assert!(flags.get().0);
            assert!(flags.get().1);
        }
        Err(err) => panic!("{:?}", err),
    }
    flags.set((false, false));

    match cli.exec_line("cmd -1 -2") {
        Ok(_) => {
            assert!(flags.get().0);
            assert!(flags.get().1);
        }
        Err(err) => panic!("{:?}", err),
    }
    flags.set((false, false));

    match cli.exec_line("cmd -12") {
        Ok(_) => {
            assert!(flags.get().0);
            assert!(flags.get().1);
        }
        Err(err) => panic!("{:?}", err),
    }
    flags.set((false, false));
}

#[test]
fn command_with_int_param_no_value() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("int")
                        .value_type(ArgType::Int)
                        .alias("i"),
                )
                .handler(|_| {
                    panic!("handler must no execute");
                }),
        )
        .build();

    match cli.exec_line("cmd --int") {
        Ok(_) => panic!("error expected"),
        Err(err) => match err.kind() {
            crate::error::Kind::ParameterValueMissed => {}
            _ => panic!("Wrong error: {:?}", err),
        },
    }

    match cli.exec_line("cmd -i") {
        Ok(_) => panic!("error expected"),
        Err(err) => match err.kind() {
            crate::error::Kind::ParameterValueMissed => {}
            _ => panic!("Wrong error: {:?}", err),
        },
    }
}

#[test]
fn command_with_int_param() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("int")
                        .value_type(ArgType::Int)
                        .alias("i")
                        .alias("ii"),
                )
                .handler(|ctx| {
                    if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("int") {
                        assert_eq!("int", param.name.as_str());
                        if let ArgValue::Int(v) = arg {
                            assert_eq!(*v, 42_i64);
                        }
                    } else {
                        panic!("parameter not found")
                    };
                }),
        )
        .build();

    match cli.exec_line("cmd --int 42") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd --ii 42") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -i 42") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_two_int_param() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("int1")
                        .value_type(ArgType::Int)
                        .alias("1"),
                )
                .parameter(
                    Parameter::with_name("int2")
                        .value_type(ArgType::Int)
                        .alias("2"),
                )
                .handler(|ctx| {
                    if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("int1") {
                        assert_eq!("int1", param.name.as_str());
                        if let ArgValue::Int(v) = arg {
                            assert_eq!(*v, 42_i64);
                        }
                    } else if let Some((param, arg)) =
                        &ctx.units.last().unwrap().parameters.get("int2")
                    {
                        assert_eq!("int2", param.name.as_str());
                        if let ArgValue::Int(v) = arg {
                            assert_eq!(*v, 333_i64);
                        }
                    } else {
                        panic!("parameter not found")
                    };
                }),
        )
        .build();

    match cli.exec_line("cmd --int1 42 --int2 333") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -1 42 -2 333") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_float_param_no_value() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("float")
                        .value_type(ArgType::Float)
                        .alias("f"),
                )
                .handler(|_| {
                    panic!("handler must no execute");
                }),
        )
        .build();

    match cli.exec_line("cmd --float") {
        Ok(_) => panic!("error expected"),
        Err(err) => match err.kind() {
            crate::error::Kind::ParameterValueMissed => {}
            _ => panic!("Wrong error: {:?}", err),
        },
    }

    match cli.exec_line("cmd -f") {
        Ok(_) => panic!("error expected"),
        Err(err) => match err.kind() {
            crate::error::Kind::ParameterValueMissed => {}
            _ => panic!("Wrong error: {:?}", err),
        },
    }
}

#[test]
fn command_with_float_param() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("float")
                        .value_type(ArgType::Float)
                        .alias("f")
                        .alias("ff"),
                )
                .handler(|ctx| {
                    if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("float") {
                        assert_eq!("float", param.name.as_str());
                        if let ArgValue::Float(v) = arg {
                            assert_eq!(*v, 4.2_f64);
                        }
                    } else {
                        panic!("parameter not found")
                    };
                }),
        )
        .build();

    match cli.exec_line("cmd --float 4.2") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd --ff 4.2") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -f 4.2") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_two_float_param() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("float1")
                        .value_type(ArgType::Float)
                        .alias("1"),
                )
                .parameter(
                    Parameter::with_name("float2")
                        .value_type(ArgType::Float)
                        .alias("2"),
                )
                .handler(|ctx| {
                    if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("float1")
                    {
                        assert_eq!("float1", param.name.as_str());
                        if let ArgValue::Float(v) = arg {
                            assert_eq!(*v, 4.2_f64);
                        }
                    } else if let Some((param, arg)) =
                        &ctx.units.last().unwrap().parameters.get("float2")
                    {
                        assert_eq!("float2", param.name.as_str());
                        if let ArgValue::Float(v) = arg {
                            assert_eq!(*v, 3.33_f64);
                        }
                    } else {
                        panic!("parameter not found")
                    };
                }),
        )
        .build();

    match cli.exec_line("cmd --float1 4.2 --float2 3.33") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -1 4.2 -2 3.33") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_string_param_no_value() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("string")
                        .value_type(ArgType::String)
                        .alias("s"),
                )
                .handler(|_| {
                    panic!("handler must no execute");
                }),
        )
        .build();

    match cli.exec_line("cmd --string") {
        Ok(_) => panic!("error expected"),
        Err(err) => match err.kind() {
            crate::error::Kind::ParameterValueMissed => {}
            _ => panic!("Wrong error: {:?}", err),
        },
    }

    match cli.exec_line("cmd -s") {
        Ok(_) => panic!("error expected"),
        Err(err) => match err.kind() {
            crate::error::Kind::ParameterValueMissed => {}
            _ => panic!("Wrong error: {:?}", err),
        },
    }
}

#[test]
fn command_with_sting_param() {
    let cli = <Cli<Test<String>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("string")
                        .value_type(ArgType::String)
                        .alias("s")
                        .alias("ss"),
                )
                .handler(|ctx| {
                    if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("string")
                    {
                        assert_eq!("string", param.name.as_str());
                        if let ArgValue::String(v) = arg {
                            return v.clone();
                        }
                    } else {
                        panic!("parameter not found")
                    };
                    String::new()
                }),
        )
        .build();

    match cli.exec_line("cmd --string abc") {
        Ok(s) => assert_eq!(s, "abc"),
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd --ss abc") {
        Ok(s) => assert_eq!(s, "abc"),
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -s abc") {
        Ok(s) => assert_eq!(s, "abc"),
        Err(err) => panic!("{:?}", err),
    }

    //quotes
    match cli.exec_line("cmd --string \"abc 123\"") {
        Ok(s) => assert_eq!(s, "abc 123"),
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd --ss \"abc 123\"") {
        Ok(s) => assert_eq!(s, "abc 123"),
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -s \"abc 123\"") {
        Ok(s) => assert_eq!(s, "abc 123"),
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_two_string_param() {
    let cli = <Cli<Test<(Option<String>, Option<String>)>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("string1")
                        .value_type(ArgType::String)
                        .alias("1"),
                )
                .parameter(
                    Parameter::with_name("string2")
                        .value_type(ArgType::String)
                        .alias("2"),
                )
                .handler(|ctx| {
                    let mut result = (None, None);
                    if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("string1")
                    {
                        assert_eq!("string1", param.name.as_str());
                        if let ArgValue::String(v) = arg {
                            result.0 = Some(v.clone());
                        }
                    };

                    if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("string2")
                    {
                        assert_eq!("string2", param.name.as_str());
                        if let ArgValue::String(v) = arg {
                            result.1 = Some(v.clone());
                        }
                    }

                    result
                }),
        )
        .build();

    match cli.exec_line("cmd --string1 4.2 --string2 3.33") {
        Ok(r) => {
            if let (Some(s1), Some(s2)) = r {
                assert_eq!(s1.as_str(), "4.2");
                assert_eq!(s2.as_str(), "3.33");
            } else {
                panic!("parameter not found {:?}", r)
            }
        }
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -1 4.2 -2 3.33") {
        Ok(r) => {
            if let (Some(s1), Some(s2)) = r {
                assert_eq!(s1.as_str(), "4.2");
                assert_eq!(s2.as_str(), "3.33");
            } else {
                panic!("parameter not found {:?}", r)
            }
        }
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd --string1 '4.2 2.4' --string2 '3.33 mm'") {
        Ok(r) => {
            if let (Some(s1), Some(s2)) = r {
                assert_eq!(s1.as_str(), "4.2 2.4");
                assert_eq!(s2.as_str(), "3.33 mm");
            } else {
                panic!("parameter not found {:?}", r)
            }
        }
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -1 \"4.2 2.4\" -2 \"3.33 mm\"") {
        Ok(r) => {
            if let (Some(s1), Some(s2)) = r {
                assert_eq!(s1.as_str(), "4.2 2.4");
                assert_eq!(s2.as_str(), "3.33 mm");
            } else {
                panic!("parameter not found {:?}", r)
            }
        }
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_mixed_params() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("bool")
                        .value_type(ArgType::Bool)
                        .alias("b")
                        .alias("bb"),
                )
                .parameter(
                    Parameter::with_name("int")
                        .value_type(ArgType::Int)
                        .alias("i")
                        .alias("ii"),
                )
                .parameter(
                    Parameter::with_name("float")
                        .value_type(ArgType::Float)
                        .alias("f")
                        .alias("ff"),
                )
                .parameter(
                    Parameter::with_name("string")
                        .value_type(ArgType::String)
                        .alias("s")
                        .alias("ss"),
                )
                .handler(|ctx| {
                    let mut expect_params = HashSet::new();
                    expect_params.insert("bool");
                    expect_params.insert("int");
                    expect_params.insert("float");
                    expect_params.insert("string");

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

                    if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("string")
                    {
                        assert_eq!("string", param.name.as_str());
                        if let ArgValue::String(v) = arg {
                            assert_eq!(*v, "bla");
                            expect_params.remove("string");
                        }
                    }

                    if !expect_params.is_empty() {
                        panic!("parameters not found: {:?}", expect_params)
                    };
                }),
        )
        .build();

    match cli.exec_line("cmd --bool --int 42 --float 4.2 --string bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd --float 4.2 --int 42 --bool --string bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd --bb --ii 42 --ff 4.2 --ss bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -b -i 42 -f 4.2 -s bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -bifs 42 4.2 bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_subcommand() {
    let is_triggered = Rc::new(Cell::new(false));
    let is_triggered_closure = is_triggered.clone();

    let cli = <Cli<Test<()>>>::builder()
        .command(CommandBuilder::with_name("cmd").subcommand(
            CommandBuilder::with_name("sub").handler(move |_| {
                is_triggered_closure.set(true);
            }),
        ))
        .build();

    if cli.exec_line("cmd").is_ok() {
        panic!("error expected")
    }
    assert!(!is_triggered.get());

    if let Err(e) = cli.exec_line("cmd sub") {
        panic!("{:?}", e)
    }
    assert!(is_triggered.get());
}

#[test]
fn command_with_subcommand_with_mixed_params() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd").subcommand(
                CommandBuilder::with_name("sub")
                    .parameter(
                        Parameter::with_name("bool")
                            .value_type(ArgType::Bool)
                            .alias("b")
                            .alias("bb"),
                    )
                    .parameter(
                        Parameter::with_name("int")
                            .value_type(ArgType::Int)
                            .alias("i")
                            .alias("ii"),
                    )
                    .parameter(
                        Parameter::with_name("float")
                            .value_type(ArgType::Float)
                            .alias("f")
                            .alias("ff"),
                    )
                    .parameter(
                        Parameter::with_name("string")
                            .value_type(ArgType::String)
                            .alias("s")
                            .alias("ss"),
                    )
                    .handler(|ctx| {
                        let mut expect_params = HashSet::new();
                        expect_params.insert("bool");
                        expect_params.insert("int");
                        expect_params.insert("float");
                        expect_params.insert("string");

                        if let Some((param, arg)) =
                            &ctx.units.last().unwrap().parameters.get("bool")
                        {
                            assert_eq!("bool", param.name.as_str());
                            if let ArgValue::Bool(v) = arg {
                                assert!(*v);
                                expect_params.remove("bool");
                            }
                        }

                        if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("int")
                        {
                            assert_eq!("int", param.name.as_str());
                            if let ArgValue::Int(v) = arg {
                                assert_eq!(*v, 42_i64);
                                expect_params.remove("int");
                            }
                        }

                        if let Some((param, arg)) =
                            &ctx.units.last().unwrap().parameters.get("float")
                        {
                            assert_eq!("float", param.name.as_str());
                            if let ArgValue::Float(v) = arg {
                                assert_eq!(*v, 4.2_f64);
                                expect_params.remove("float");
                            }
                        }

                        if let Some((param, arg)) =
                            &ctx.units.last().unwrap().parameters.get("string")
                        {
                            assert_eq!("string", param.name.as_str());
                            if let ArgValue::String(v) = arg {
                                assert_eq!(*v, "bla");
                                expect_params.remove("string");
                            }
                        }

                        if !expect_params.is_empty() {
                            panic!("parameters not found: {:?}", expect_params)
                        };
                    }),
            ),
        )
        .build();

    match cli.exec_line("cmd sub --bool --int 42 --float 4.2 --string bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd sub --bb --ii 42 --ff 4.2 --ss bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd sub -b -i 42 -f 4.2 -s bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd sub -bifs 42 4.2 bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_mixed_params_with_subcommand() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("bool")
                        .value_type(ArgType::Bool)
                        .alias("b")
                        .alias("bb"),
                )
                .parameter(
                    Parameter::with_name("int")
                        .value_type(ArgType::Int)
                        .alias("i")
                        .alias("ii"),
                )
                .parameter(
                    Parameter::with_name("float")
                        .value_type(ArgType::Float)
                        .alias("f")
                        .alias("ff"),
                )
                .parameter(
                    Parameter::with_name("string")
                        .value_type(ArgType::String)
                        .alias("s")
                        .alias("ss"),
                )
                .subcommand(CommandBuilder::with_name("sub").handler(|ctx| {
                    let mut expect_params = HashSet::new();
                    expect_params.insert("bool");
                    expect_params.insert("int");
                    expect_params.insert("float");
                    expect_params.insert("string");

                    assert_eq!(ctx.units.len(), 3);
                    let parameters = &ctx.units[1].parameters;

                    if let Some((param, arg)) = parameters.get("bool") {
                        assert_eq!("bool", param.name.as_str());
                        if let ArgValue::Bool(v) = arg {
                            assert!(*v);
                            expect_params.remove("bool");
                        }
                    }

                    if let Some((param, arg)) = parameters.get("int") {
                        assert_eq!("int", param.name.as_str());
                        if let ArgValue::Int(v) = arg {
                            assert_eq!(*v, 42_i64);
                            expect_params.remove("int");
                        }
                    }

                    if let Some((param, arg)) = parameters.get("float") {
                        assert_eq!("float", param.name.as_str());
                        if let ArgValue::Float(v) = arg {
                            assert_eq!(*v, 4.2_f64);
                            expect_params.remove("float");
                        }
                    }

                    if let Some((param, arg)) = parameters.get("string") {
                        assert_eq!("string", param.name.as_str());
                        if let ArgValue::String(v) = arg {
                            assert_eq!(*v, "bla");
                            expect_params.remove("string");
                        }
                    }

                    if !expect_params.is_empty() {
                        panic!("parameters not found: {:?}", expect_params)
                    };
                })),
        )
        .build();

    match cli.exec_line("cmd --bool --int 42 --float 4.2 --string bla sub") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd --ii 42 --ff 4.2 --bb --ss bla sub") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -b -i 42 -f 4.2 -s bla sub") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -bifs 42 4.2 bla sub") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_mixed_params_with_subcommand_with_mixed_params() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .parameter(
                    Parameter::with_name("bool")
                        .value_type(ArgType::Bool)
                        .alias("b")
                        .alias("bb"),
                )
                .parameter(
                    Parameter::with_name("int")
                        .value_type(ArgType::Int)
                        .alias("i")
                        .alias("ii"),
                )
                .parameter(
                    Parameter::with_name("float")
                        .value_type(ArgType::Float)
                        .alias("f")
                        .alias("ff"),
                )
                .parameter(
                    Parameter::with_name("string")
                        .value_type(ArgType::String)
                        .alias("s")
                        .alias("ss"),
                )
                .subcommand(
                    CommandBuilder::with_name("sub")
                        .parameter(
                            Parameter::with_name("bool")
                                .value_type(ArgType::Bool)
                                .alias("b")
                                .alias("bb"),
                        )
                        .parameter(
                            Parameter::with_name("int")
                                .value_type(ArgType::Int)
                                .alias("i")
                                .alias("ii"),
                        )
                        .parameter(
                            Parameter::with_name("float")
                                .value_type(ArgType::Float)
                                .alias("f")
                                .alias("ff"),
                        )
                        .parameter(
                            Parameter::with_name("string")
                                .value_type(ArgType::String)
                                .alias("s")
                                .alias("ss"),
                        )
                        .handler(|ctx| {
                            let mut expect_params_cmd = HashSet::new();
                            expect_params_cmd.insert("bool");
                            expect_params_cmd.insert("int");
                            expect_params_cmd.insert("float");
                            expect_params_cmd.insert("string");
                            let mut expect_params_sub = expect_params_cmd.clone();

                            assert_eq!(ctx.units.len(), 3);

                            let parent_parameters = &ctx.units[1].parameters;

                            if let Some((param, arg)) = parent_parameters.get("bool") {
                                assert_eq!("bool", param.name.as_str());
                                if let ArgValue::Bool(v) = arg {
                                    assert!(*v);
                                    expect_params_cmd.remove("bool");
                                }
                            }

                            if let Some((param, arg)) = parent_parameters.get("int") {
                                assert_eq!("int", param.name.as_str());
                                if let ArgValue::Int(v) = arg {
                                    assert_eq!(*v, 42_i64);
                                    expect_params_cmd.remove("int");
                                }
                            }

                            if let Some((param, arg)) = parent_parameters.get("float") {
                                assert_eq!("float", param.name.as_str());
                                if let ArgValue::Float(v) = arg {
                                    assert_eq!(*v, 4.2_f64);
                                    expect_params_cmd.remove("float");
                                }
                            }

                            if let Some((param, arg)) = parent_parameters.get("string") {
                                assert_eq!("string", param.name.as_str());
                                if let ArgValue::String(v) = arg {
                                    assert_eq!(*v, "bla");
                                    expect_params_cmd.remove("string");
                                }
                            }

                            let parameters = &ctx.units[2].parameters;

                            if let Some((param, arg)) = parameters.get("bool") {
                                assert_eq!("bool", param.name.as_str());
                                if let ArgValue::Bool(v) = arg {
                                    assert!(*v);
                                    expect_params_sub.remove("bool");
                                }
                            }

                            if let Some((param, arg)) = parameters.get("int") {
                                assert_eq!("int", param.name.as_str());
                                if let ArgValue::Int(v) = arg {
                                    assert_eq!(*v, 24_i64);
                                    expect_params_sub.remove("int");
                                }
                            }

                            if let Some((param, arg)) = parameters.get("float") {
                                assert_eq!("float", param.name.as_str());
                                if let ArgValue::Float(v) = arg {
                                    assert_eq!(*v, 2.4_f64);
                                    expect_params_sub.remove("float");
                                }
                            }

                            if let Some((param, arg)) = parameters.get("string") {
                                assert_eq!("string", param.name.as_str());
                                if let ArgValue::String(v) = arg {
                                    assert_eq!(*v, "alb");
                                    expect_params_sub.remove("string");
                                }
                            }

                            if !expect_params_cmd.is_empty() {
                                panic!("parameters not found for cmd: {:?}", expect_params_cmd)
                            };

                            if !expect_params_sub.is_empty() {
                                panic!("parameters not found for sub: {:?}", expect_params_sub)
                            };
                        }),
                ),
        )
        .build();

    match cli.exec_line(
        "cmd --bool --int 42 --float 4.2 --string bla sub --float 2.4 --int 24 --bool --string alb",
    ) {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd --ii 42 --ff 4.2 --bb --ss bla sub --ff 2.4 --ii 24 --bb --ss alb") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -b -i 42 -f 4.2 -s bla sub -f 2.4 -i 24 -b -s alb") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -bifs 42 4.2 bla sub -fibs 2.4 24 alb") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_bool_value() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .use_value(ArgType::Bool)
                .handler(|ctx| {
                    if let Some(value) = ctx.command_units().last().unwrap().value() {
                        match value {
                            ArgValue::Bool(_) => {}
                            _ => panic!("bool value expected"),
                        }
                    } else {
                        panic!("value expected");
                    }
                }),
        )
        .build();

    match cli.exec_line("cmd true") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_int_value() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .use_value(ArgType::Int)
                .handler(|ctx| {
                    if let Some(value) = ctx.command_units().last().unwrap().value() {
                        match value {
                            ArgValue::Int(v) => assert_eq!(v, &42_i64),
                            _ => panic!("int value expected"),
                        }
                    } else {
                        panic!("value expected");
                    }
                }),
        )
        .build();

    match cli.exec_line("cmd 42") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_float_value() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .use_value(ArgType::Float)
                .handler(|ctx| {
                    if let Some(value) = ctx.command_units().last().unwrap().value() {
                        match value {
                            ArgValue::Float(v) => assert_eq!(v, &4.2_f64),
                            _ => panic!("float value expected"),
                        }
                    } else {
                        panic!("value expected");
                    }
                }),
        )
        .build();

    match cli.exec_line("cmd 4.2") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_string_value() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .use_value(ArgType::String)
                .handler(|ctx| {
                    if let Some(value) = ctx.command_units().last().unwrap().value() {
                        match value {
                            ArgValue::String(v) => assert_eq!(v, "bla"),
                            _ => panic!("string value expected"),
                        }
                    } else {
                        panic!("value expected");
                    }
                }),
        )
        .build();

    match cli.exec_line("cmd bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_with_mixed_params_and_value() {
    let cli = <Cli<Test<()>>>::builder()
        .command(
            CommandBuilder::with_name("cmd")
                .use_value(ArgType::Bool)
                .parameter(
                    Parameter::with_name("bool")
                        .value_type(ArgType::Bool)
                        .alias("b")
                        .alias("bb"),
                )
                .parameter(
                    Parameter::with_name("int")
                        .value_type(ArgType::Int)
                        .alias("i")
                        .alias("ii"),
                )
                .parameter(
                    Parameter::with_name("float")
                        .value_type(ArgType::Float)
                        .alias("f")
                        .alias("ff"),
                )
                .parameter(
                    Parameter::with_name("string")
                        .value_type(ArgType::String)
                        .alias("s")
                        .alias("ss"),
                )
                .handler(|ctx| {
                    let mut expect_params = HashSet::new();
                    expect_params.insert("bool");
                    expect_params.insert("int");
                    expect_params.insert("float");
                    expect_params.insert("string");
                    expect_params.insert("value");

                    if let Some(ArgValue::Bool(v)) = &ctx.units.last().unwrap().value() {
                        assert!(!*v);
                        expect_params.remove("value");
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

                    if let Some((param, arg)) = &ctx.units.last().unwrap().parameters.get("string")
                    {
                        assert_eq!("string", param.name.as_str());
                        if let ArgValue::String(v) = arg {
                            assert_eq!(*v, "bla");
                            expect_params.remove("string");
                        }
                    }

                    if !expect_params.is_empty() {
                        panic!("parameters not found: {:?}", expect_params)
                    };
                }),
        )
        .build();

    match cli.exec_line("cmd false --bool --int 42 --float 4.2 --string bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd --float 4.2 --int 42 --bool --string bla off") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd 0 --bb --ii 42 --ff 4.2 --ss bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -b -i 42 no -f 4.2 -s bla") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }

    match cli.exec_line("cmd -bifs 42 4.2 bla off") {
        Ok(_) => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn command_help() {
    let help_text = Rc::new(RefCell::new(String::new()));
    let printer = TestPrinter(help_text.clone());
    let cli = <Cli<Test<()>>>::builder()
        .set_printer(printer)
        .print_help(true)
        .command(
            CommandBuilder::with_name("cmd")
                .use_value(ArgType::Bool)
                .parameter(
                    Parameter::with_name("bool")
                        .value_type(ArgType::Bool)
                        .alias("b")
                        .alias("bb"),
                )
                .parameter(
                    Parameter::with_name("int")
                        .value_type(ArgType::Int)
                        .alias("i")
                        .alias("ii"),
                ),
        )
        .command(CommandBuilder::with_name("another_cmd").use_value(ArgType::Bool))
        .build();

    assert!(cli.exec_line("help").is_ok());
    assert_eq!(
        help_text.borrow().as_str(),
        r"Help:
  Subcommands:
    another_cmd          
    cmd                  
    help                 This help"
    );
}

#[test]
fn sub_command_help() {
    let help_text = Rc::new(RefCell::new(String::new()));
    let printer = TestPrinter(help_text.clone());
    let cli = <Cli<Test<()>>>::builder()
        .set_printer(printer)
        .print_help(true)
        .command(
            CommandBuilder::with_name("cmd")
                .use_value(ArgType::Bool)
                .parameter(
                    Parameter::with_name("bool")
                        .value_type(ArgType::Bool)
                        .alias("b")
                        .alias("bb")
                        .description("Boolean param"),
                )
                .parameter(
                    Parameter::with_name("int")
                        .value_type(ArgType::Int)
                        .alias("i")
                        .alias("ii")
                        .description("Integer param"),
                )
                .subcommand(
                    CommandBuilder::with_name("sub")
                        .use_value(ArgType::Int)
                        .description("sub command"),
                ),
        )
        .command(CommandBuilder::with_name("another_cmd").use_value(ArgType::Bool))
        .build();

    assert!(cli.exec_line("cmd help").is_ok());
    assert_eq!(
        help_text.borrow().as_str(),
        r"Help:
  Parameters:
    --bool,-b,--bb      <bool>  Boolean param
    --int,-i,--ii       <int>   Integer param
----------------------------------------
  Subcommands:
    help                 This help
    sub                  sub command"
    );
}

#[test]
fn sub_command() {
    let cli = <Cli<Test<bool>>>::builder()
        .print_help(true)
        .command(
            CommandBuilder::with_name("cmd")
                .use_value(ArgType::Bool)
                .subcommand(
                    CommandBuilder::with_name("sub")
                        .use_value(ArgType::Int)
                        .handler(|_| true),
                ),
        )
        .command(CommandBuilder::with_name("another_cmd").use_value(ArgType::Bool))
        .build();

    let res = cli.exec_line("cmd sub 10");
    assert!(res.is_ok());
    assert!(res.unwrap());
}
