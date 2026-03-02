#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use criterion::{criterion_group, criterion_main, Criterion};
use diffutilslib::{
    arg_parser::ArgParser,
    cmp::params_cmp::{ParamsCmp, ParamsCmpOk},
};
use std::{ffi::OsString, time::Duration};

const WARM_UP_TIME_MS: u64 = 500;
const MEASUREMENT_TIME_MS: u64 = 2000;

criterion_group!(benches, bench_parser,);
criterion_main!(benches);

// All results are a few microseconds, so negligible.
fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bench Parser");

    group.warm_up_time(Duration::from_millis(WARM_UP_TIME_MS));
    // group.measurement_time(Duration::from_millis(MEASUREMENT_TIME_MS));
    group.sample_size(10);

    group.bench_function("Parse bytes Exabyte", |b| {
        b.iter(|| ArgParser::parse_bytes("1EIB"))
    });

    // group.bench_function("Parse short option", |b| {
    //     b.iter(|| parse_single_arg("cmd file_1.txt file_2.txt -b -l"))
    // });
    // group.bench_function("Parse long option", |b| {
    //     b.iter(|| parse_single_arg("cmd file_1.txt file_2.txt --print-bytes --verbose"))
    // });
    // group.bench_function("Parse ignore bytes", |b| {
    //     b.iter(|| parse_single_arg("cmd file_1.txt file_2.txt 100KiB 1MiB"))
    // });
    group.bench_function("Parse all", |b| {
        b.iter(|| {
            parse_single_arg("cmd file_1.txt file_2.txt -bl n10M --ignore-initial=100KiB:1MiB")
        })
    });
    group.bench_function("Parse error", |b| {
        b.iter(|| parse_single_arg("cmd file_1.txt file_2.txt --something-unknown"))
    });
    group.bench_function("Parse help", |b| b.iter(|| parse_single_arg("cmd --help")));

    group.finish();
}

fn parse_single_arg(cmd: &str) -> String {
    let args = str_to_options(cmd).into_iter().peekable();
    let params = match ParamsCmp::parse_params(args) {
        Ok(res) => match res {
            ParamsCmpOk::Info(info) => {
                // println!("{info}");
                // return ExitCode::from(0);
                return info.to_string();
            }
            ParamsCmpOk::ParamsCmp(params) => params,
        },
        Err(e) => {
            // eprintln!("{e}");
            // return ExitCode::from(2);
            return e.to_string();
        }
    };
    return params.file_1.to_string_lossy().to_string();
}

fn str_to_options(opt: &str) -> Vec<OsString> {
    let s: Vec<OsString> = opt
        .split(" ")
        .into_iter()
        .map(|s| OsString::from(s))
        .collect();

    s
}
