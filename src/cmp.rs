// This file is part of the uutils diffutils package.
//
// For the full copyright and license information, please view the LICENSE-*
// files that was distributed with this source code.

/// TODO General Topics
/// - ParamsGen document
/// - ParamsGen EXE_NAME depending on app
/// - arg default
/// - PR 183, branch u64/u128
/// - branch bench
/// - branch tests
///   - $ cargo run -- cmp old.txt new.txt -n1 other result
///   - $ cargo run -- cmp old.txt new.txt -bln50
/// - integration.rs: adjust to new error messages
///
/// Requirements Param
/// - Return String for Help but mark as OK. Probably Enum String or Params.
/// - Separation of concerns, no output or exit of the app
/// - Reusable components
// pub mod params;
pub mod params_cmp;
use crate::cmp::params_cmp::{ParamsCmp, ParamsCmpOk};
use crate::utils::format_failure_to_read_input_file;
use std::env::{self, ArgsOs};
use std::ffi::OsString;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::iter::Peekable;
use std::process::ExitCode;
use std::{cmp, fs, io};

pub const EXE_NAME: &str = "cmp";
/// for --bytes, so really large number limits can be expressed, like 1Y.
#[cfg(not(feature = "cmp_bytes_limit_128_bit"))]
pub type Bytes = u64;
#[cfg(feature = "cmp_bytes_limit_128_bit")]
pub type Bytes = u128;
// ignore initial is currently limited to u64, as take(skip) is used.
pub type IgnInit = u64;

fn prepare_reader(
    path: &OsString,
    skip: &Option<IgnInit>,
    params: &ParamsCmp,
) -> Result<Box<dyn BufRead>, String> {
    let mut reader: Box<dyn BufRead> = if path == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        match fs::File::open(path) {
            Ok(file) => Box::new(BufReader::new(file)),
            Err(e) => {
                return Err(format_failure_to_read_input_file(
                    &params.util.to_os_string(),
                    path,
                    &e,
                ));
            }
        }
    };

    if let Some(skip) = skip {
        if let Err(e) = io::copy(&mut reader.by_ref().take(*skip), &mut io::sink()) {
            return Err(format_failure_to_read_input_file(
                &params.util.to_os_string(),
                path,
                &e,
            ));
        }
    }

    Ok(reader)
}

#[derive(Debug)]
pub enum Cmp {
    Equal,
    Different,
}

/// This is the main function to compare the files. \
/// Files are limited to u64 bytes and u64 lines.
// TODO CmpError
pub fn cmp(params: &ParamsCmp) -> Result<Cmp, String> {
    let mut from = prepare_reader(&params.file_1, &params.ignore_initial_bytes_file_1, params)?;
    let mut to = prepare_reader(&params.file_2, &params.ignore_initial_bytes_file_2, params)?;

    let mut offset_width = params.bytes_limit.unwrap_or(Bytes::MAX);

    if let (Ok(a_meta), Ok(b_meta)) = (fs::metadata(&params.file_1), fs::metadata(&params.file_2)) {
        #[cfg(not(target_os = "windows"))]
        let (a_size, b_size) = (a_meta.size(), b_meta.size());

        #[cfg(target_os = "windows")]
        let (a_size, b_size) = (a_meta.file_size(), b_meta.file_size());

        // If the files have different sizes, we already know they are not identical. If we have not
        // been asked to show even the first difference, we can quit early.
        if params.silent && a_size != b_size {
            return Ok(Cmp::Different);
        }

        let smaller = cmp::min(a_size, b_size) as Bytes;
        offset_width = cmp::min(smaller, offset_width);
    }

    let offset_width = 1 + offset_width.checked_ilog10().unwrap_or(1) as usize;

    // Capacity calc: at_byte width + 2 x 3-byte octal numbers + 2 x 4-byte value + 4 spaces
    let mut output = Vec::<u8>::with_capacity(offset_width + 3 * 2 + 4 * 2 + 4);

    let mut at_byte: Bytes = 1;
    let mut at_line: u64 = 1;
    let mut start_of_line = true;
    let mut stdout = BufWriter::new(io::stdout().lock());
    let mut compare = Cmp::Equal;
    loop {
        // Fill up our buffers.
        let from_buf = match from.fill_buf() {
            Ok(buf) => buf,
            Err(e) => {
                return Err(format_failure_to_read_input_file(
                    &params.util.to_os_string(),
                    &params.file_1,
                    &e,
                ));
            }
        };

        let to_buf = match to.fill_buf() {
            Ok(buf) => buf,
            Err(e) => {
                return Err(format_failure_to_read_input_file(
                    &params.util.to_os_string(),
                    &params.file_2,
                    &e,
                ));
            }
        };

        // Check for EOF conditions.
        if from_buf.is_empty() && to_buf.is_empty() {
            break;
        }

        if from_buf.is_empty() || to_buf.is_empty() {
            let eof_on = if from_buf.is_empty() {
                &params.file_1.to_string_lossy()
            } else {
                &params.file_2.to_string_lossy()
            };

            report_eof(at_byte, at_line, start_of_line, eof_on, params);
            return Ok(Cmp::Different);
        }

        // Fast path - for long files in which almost all bytes are the same we
        // can do a direct comparison to let the compiler optimize.
        let consumed = std::cmp::min(from_buf.len(), to_buf.len());
        if from_buf[..consumed] == to_buf[..consumed] {
            let last = from_buf[..consumed].last().unwrap();

            // Unclear if this is necessary to prevent errors if the file is larger than Bytes::MAX.
            // Will have an performance impact.
            // if let None = at_byte.checked_add(consumed as Bytes) {
            //     panic!("File larger than {} bytes.", Bytes::MAX);
            // };
            at_byte += consumed as Bytes;
            at_line += from_buf[..consumed].iter().filter(|&c| *c == b'\n').count() as u64;

            start_of_line = *last == b'\n';

            if let Some(bytes_limit) = params.bytes_limit {
                if at_byte > bytes_limit {
                    break;
                }
            }

            from.consume(consumed);
            to.consume(consumed);

            continue;
        }

        // Iterate over the buffers, the zip iterator will stop us as soon as the
        // first one runs out.
        for (&from_byte, &to_byte) in from_buf.iter().zip(to_buf.iter()) {
            if from_byte != to_byte {
                compare = Cmp::Different;

                if params.verbose {
                    format_verbose_difference(
                        from_byte,
                        to_byte,
                        at_byte,
                        offset_width,
                        &mut output,
                        params,
                    )?;
                    stdout
                        .write_all(output.as_slice())
                        .map_err(|e| format!("{}: error printing output: {e}", params.util))?;
                    output.clear();
                } else {
                    report_difference(from_byte, to_byte, at_byte, at_line, params);
                    return Ok(Cmp::Different);
                }
            }

            start_of_line = from_byte == b'\n';
            if start_of_line {
                at_line += 1;
            }

            at_byte += 1;

            if let Some(max_bytes) = params.bytes_limit {
                if at_byte > max_bytes {
                    break;
                }
            }
        }

        // Notify our readers about the bytes we went over.
        from.consume(consumed);
        to.consume(consumed);
    }

    Ok(compare)
}

/// Entry into cmp.
///
/// Param options, e.g. 'cmp file1.txt file2.txt -bd n2000kB'.
/// - Program name - Usually 'cmp' as first parameter, but accept any OsString.
/// - cmp options - as documented at <https://www.gnu.org/software/diffutils/manual/html_node/cmp-Options.html>
// Exit codes are documented at
// https://www.gnu.org/software/diffutils/manual/html_node/Invoking-cmp.html
//     An exit status of 0 means no differences were found,
//     1 means some differences were found,
//     and 2 means trouble.
// TODO first param util: DiffUtility,
pub fn main(opts: Peekable<ArgsOs>) -> ExitCode {
    // let params = match Params::parse_params(options) {
    //     Ok(res) => match res {
    //         ParamsParseOk::Info(info) => {
    //             println!("{info}");
    //             return ExitCode::from(0);
    //         }
    //         ParamsParseOk::Params(params) => params,
    //     },
    //     Err(e) => {
    //         eprintln!("{e}");
    //         return ExitCode::from(2);
    //     }
    // };
    let params = match ParamsCmp::parse_params(opts) {
        Ok(res) => match res {
            ParamsCmpOk::Info(info) => {
                println!("{info}");
                return ExitCode::from(0);
            }
            ParamsCmpOk::ParamsCmp(params) => params,
        },
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };

    if params.file_1 == "-" && params.file_2 == "-"
        || same_file::is_same_file(&params.file_1, &params.file_2).unwrap_or(false)
    {
        return ExitCode::SUCCESS;
    }

    match cmp(&params) {
        Ok(Cmp::Equal) => ExitCode::SUCCESS,
        Ok(Cmp::Different) => ExitCode::from(1),
        Err(e) => {
            if !params.silent {
                eprintln!("{e}");
            }
            ExitCode::from(2)
        }
    }
}

#[inline]
fn format_octal(byte: u8, buf: &mut [u8; 3]) -> &str {
    *buf = [b' ', b' ', b'0'];

    let mut num = byte;
    let mut idx = 2; // Start at the last position in the buffer

    // Generate octal digits
    while num > 0 {
        buf[idx] = b'0' + num % 8;
        num /= 8;
        idx = idx.saturating_sub(1);
    }

    // SAFETY: the operations we do above always land within ascii range.
    unsafe { std::str::from_utf8_unchecked(&buf[..]) }
}

#[inline]
fn write_visible_byte(output: &mut Vec<u8>, byte: u8) -> usize {
    match byte {
        // Control characters: ^@, ^A, ..., ^_
        0..=31 => {
            output.push(b'^');
            output.push(byte + 64);
            2
        }
        // Printable ASCII (space through ~)
        32..=126 => {
            output.push(byte);
            1
        }
        // DEL: ^?
        127 => {
            output.extend_from_slice(b"^?");
            2
        }
        // High bytes with control equivalents: M-^@, M-^A, ..., M-^_
        128..=159 => {
            output.push(b'M');
            output.push(b'-');
            output.push(b'^');
            output.push(byte - 64);
            4
        }
        // High bytes: M-<space>, M-!, ..., M-~
        160..=254 => {
            output.push(b'M');
            output.push(b'-');
            output.push(byte - 128);
            3
        }
        // Byte 255: M-^?
        255 => {
            output.extend_from_slice(b"M-^?");
            4
        }
    }
}

/// Writes a byte in visible form with right-padding to 4 spaces.
#[inline]
fn write_visible_byte_padded(output: &mut Vec<u8>, byte: u8) {
    const SPACES: &[u8] = b"    ";
    const WIDTH: usize = SPACES.len();

    let display_width = write_visible_byte(output, byte);

    // Add right-padding spaces
    let padding = WIDTH.saturating_sub(display_width);
    output.extend_from_slice(&SPACES[..padding]);
}

/// Formats a byte as a visible string (for non-performance-critical path)
#[inline]
fn format_visible_byte(byte: u8) -> String {
    let mut result = Vec::with_capacity(4);
    write_visible_byte(&mut result, byte);
    // SAFETY: the checks and shifts in write_visible_byte match what cat and GNU
    // cmp do to ensure characters fall inside the ascii range.
    unsafe { String::from_utf8_unchecked(result) }
}

// This function has been optimized to not use the Rust fmt system, which
// leads to a massive speed up when processing large files: cuts the time
// for comparing 2 ~36MB completely different files in half on an M1 Max.
#[inline]
fn format_verbose_difference(
    from_byte: u8,
    to_byte: u8,
    at_byte: Bytes,
    offset_width: usize,
    output: &mut Vec<u8>,
    params: &ParamsCmp,
) -> Result<(), String> {
    assert!(!params.silent);

    let mut at_byte_buf = itoa::Buffer::new();
    let mut from_oct = [0u8; 3]; // for octal conversions
    let mut to_oct = [0u8; 3];

    if params.print_bytes {
        // "{:>width$} {:>3o} {:4} {:>3o} {}",
        let at_byte_str = at_byte_buf.format(at_byte);
        let at_byte_padding = offset_width.saturating_sub(at_byte_str.len());

        for _ in 0..at_byte_padding {
            output.push(b' ')
        }

        output.extend_from_slice(at_byte_str.as_bytes());

        output.push(b' ');

        output.extend_from_slice(format_octal(from_byte, &mut from_oct).as_bytes());

        output.push(b' ');

        write_visible_byte_padded(output, from_byte);

        output.push(b' ');

        output.extend_from_slice(format_octal(to_byte, &mut to_oct).as_bytes());

        output.push(b' ');

        write_visible_byte(output, to_byte);

        output.push(b'\n');
    } else {
        // "{:>width$} {:>3o} {:>3o}"
        let at_byte_str = at_byte_buf.format(at_byte);
        let at_byte_padding = offset_width - at_byte_str.len();

        for _ in 0..at_byte_padding {
            output.push(b' ')
        }

        output.extend_from_slice(at_byte_str.as_bytes());

        output.push(b' ');

        output.extend_from_slice(format_octal(from_byte, &mut from_oct).as_bytes());

        output.push(b' ');

        output.extend_from_slice(format_octal(to_byte, &mut to_oct).as_bytes());

        output.push(b'\n');
    }

    Ok(())
}

#[inline]
fn report_eof(at_byte: Bytes, at_line: u64, start_of_line: bool, eof_on: &str, params: &ParamsCmp) {
    if params.silent {
        return;
    }

    if at_byte == 1 {
        eprintln!("{}: EOF on '{}' which is empty", params.util, eof_on);
    } else if params.verbose {
        eprintln!(
            "{}: EOF on '{}' after byte {}",
            params.util,
            eof_on,
            at_byte - 1,
        );
    } else if start_of_line {
        eprintln!(
            "{}: EOF on '{}' after byte {}, line {}",
            params.util,
            eof_on,
            at_byte - 1,
            at_line - 1
        );
    } else {
        eprintln!(
            "{}: EOF on '{}' after byte {}, in line {}",
            params.util,
            eof_on,
            at_byte - 1,
            at_line
        );
    }
}

fn is_posix_locale() -> bool {
    let locale = if let Ok(locale) = env::var("LC_ALL") {
        locale
    } else if let Ok(locale) = env::var("LC_MESSAGES") {
        locale
    } else if let Ok(locale) = env::var("LANG") {
        locale
    } else {
        "C".to_string()
    };

    locale == "C" || locale == "POSIX"
}

#[inline]
fn report_difference(from_byte: u8, to_byte: u8, at_byte: Bytes, at_line: u64, params: &ParamsCmp) {
    if params.silent {
        return;
    }

    let term = if is_posix_locale() && !params.print_bytes {
        "char"
    } else {
        "byte"
    };
    print!(
        "{} {} differ: {term} {}, line {}",
        &params.file_1.to_string_lossy(),
        &params.file_2.to_string_lossy(),
        at_byte,
        at_line
    );
    if params.print_bytes {
        let char_width = if to_byte >= 0x7F { 2 } else { 1 };
        print!(
            " is {:>3o} {:char_width$} {:>3o} {:char_width$}",
            from_byte,
            format_visible_byte(from_byte),
            to_byte,
            format_visible_byte(to_byte)
        );
    }
    println!();
}

#[cfg(not(target_os = "windows"))]
use std::os::fd::{AsRawFd, FromRawFd};

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::MetadataExt;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

#[cfg(not(target_os = "windows"))]
fn is_stdout_dev_null() -> bool {
    let Ok(dev_null) = fs::metadata("/dev/null") else {
        return false;
    };

    let stdout_fd = io::stdout().lock().as_raw_fd();

    // SAFETY: we have exclusive access to stdout right now.
    let stdout_file = unsafe { fs::File::from_raw_fd(stdout_fd) };
    let Ok(stdout) = stdout_file.metadata() else {
        return false;
    };

    let is_dev_null = stdout.dev() == dev_null.dev() && stdout.ino() == dev_null.ino();

    // Don't let File close the fd. It's unfortunate that File doesn't have a leak_fd().
    std::mem::forget(stdout_file);

    is_dev_null
}
