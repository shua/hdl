use libfst_sys as fstapi;
use log::debug;

struct Var {
    name: Vec<String>,
    typ: fstapi::VarType,
    size: usize,
    data: Vec<(u64, String)>,
}

#[derive(Default)]
struct Fst {
    vars: std::collections::HashMap<u32, Var>,
    crit_time: Vec<u64>,
}

fn clib(
    file: &str,
    var_filter: impl Fn(&Vec<String>, &str, fstapi::VarType) -> bool,
    max_time: Option<u64>,
) -> Fst {
    let mut fst = fstapi::Reader::open(&file).expect("open fst file");
    debug!(
        "version: {}, date: {}",
        fst.version_string(),
        fst.date_string().trim()
    );
    debug!(
        "start: {}, end: {}, timescale: 10^{}, timezero: {}",
        fst.start_time(),
        fst.end_time(),
        fst.timescale(),
        fst.timezero(),
    );
    debug!(
        "value_change_section_count: {}, current_scope_len: {}, max_handle: {}",
        fst.value_change_section_count(),
        fst.current_scope_len(),
        fst.max_handle(),
    );

    debug!("\nhier:");
    let mut vars: std::collections::HashMap<fstapi::fstHandle, Var> =
        std::collections::HashMap::new();
    let mut aliases: Vec<(Vec<String>, fstapi::fstHandle)> = vec![];
    let mut scope: Vec<String> = vec![];
    let mut i = 0;
    while let Some(hier) = fst.next_hier() {
        match &hier {
            fstapi::Hier::Scope(s) => scope.push(s.name().to_string()),
            fstapi::Hier::Upscope => {
                scope.pop();
            }
            fstapi::Hier::Var(v) if var_filter(&scope, v.name(), v.typ()) => {
                let mut name = scope.clone();
                name.push(v.name().to_string());
                if v.is_alias() {
                    aliases.push((name, v.handle()));
                } else {
                    vars.insert(
                        v.handle(),
                        Var {
                            name,
                            typ: v.typ(),
                            size: v.length(),
                            data: vec![],
                        },
                    );
                }
            }
            _ => {}
        }
        debug!("{i}: {:?}", hier);
        i += 1;
    }

    debug!("\nblocks:");
    let end_time = max_time.unwrap_or(fst.end_time()).min(fst.end_time());
    let limit_keys: Vec<_> = vars.keys().copied().collect();
    let mut crit_time = vec![fst.start_time(), end_time];
    let time_range = fst.start_time()..end_time;
    fst.foreach_block(
        Some(time_range.clone()),
        Some(&limit_keys),
        |time, handle, val| {
            debug!("block: {time} {handle} {val:?}");
            if time_range.contains(&time) {
                let v = vars.get_mut(&handle).unwrap();
                crit_time.push(time);
                v.data.push((time, val.to_string()));
            }
        },
    );
    crit_time.sort();
    crit_time.dedup();

    Fst { vars, crit_time }
}

fn main() {
    env_logger::init();

    let (width, include_parms, limit_scope, file) = {
        use clap::builder::{Arg, ArgAction, Command};
        let mut cmd = Command::new("termwave")
            .arg(
                Arg::new("width")
                    .short('w')
                    .help("max width in columns of displayed data"),
            )
            .arg(
                Arg::new("parameters")
                    .short('p')
                    .help("include parameters")
                    .num_args(0)
                    .action(ArgAction::SetTrue),
            )
            .arg(Arg::new("scope").short('s').help("limit to specific scope"))
            .arg(
                Arg::new("file")
                    .num_args(1)
                    .required(true)
                    .help("fst input file"),
            );
        let args = cmd.get_matches_mut();
        let width = args.get_one::<String>("width").map(|w| w.parse::<u64>());
        let width = match width {
            Some(Err(_)) => {
                eprintln!("error: `width` must be a number");
                cmd.print_long_help().unwrap();
                return;
            }
            Some(Ok(w)) => Some(w),
            None => None,
        };
        let parameters = args.get_one::<bool>("parameters").copied().unwrap();
        let scope = args
            .get_one::<String>("scope")
            .map(|s| s.split(".").map(str::to_string).collect::<Vec<_>>());
        let file = args.get_one::<String>("file").unwrap();
        (width, parameters, scope, file.clone())
    };
    let fst = clib(
        &file,
        |scope, _name, typ| {
            (&limit_scope).as_ref().map(|s| scope == s).unwrap_or(true)
                && (include_parms || typ != fstapi::VarType::VcdParameter)
        },
        width,
    );

    {
        println!("$time");
        let mut i = fst.crit_time[0];
        for &t in &fst.crit_time {
            if t < i || t >= *fst.crit_time.last().unwrap() {
                continue;
            }
            while i < t {
                print!(" ");
                i += 1;
            }
            let label = format!("|{t}");
            print!("{label}");
            i += u64::try_from(label.len()).unwrap();
        }
        println!();
    }
    let (start_time, end_time) = (fst.crit_time[0], *fst.crit_time.last().unwrap());
    for (h, v) in &fst.vars {
        println!("{h}: {:?} {:?} {}", v.name, v.typ, v.size);
        let draw_line = |top: bool| {
            let mut vals = v.data.iter().peekable();
            let mut i = start_time;
            while i < end_time {
                while vals.peek().map(|(t, _)| *t < i).unwrap_or(false) {
                    vals.next();
                }
                if let Some(&(t, ref label)) = vals.next() {
                    let label_trimmed = label.trim_start_matches('0');
                    let space_char =
                        if (label_trimmed == "1" && !top) || (label_trimmed == "" && top) {
                            " "
                        } else {
                            "_"
                        };
                    let label_len = u64::try_from(label.len()).unwrap();
                    let next_t = vals.peek().map(|(t, _)| *t).unwrap_or(end_time);
                    let label_start =
                        (end_time - label_len).min(((next_t - t - label_len) / 2) + t);
                    while i < label_start {
                        print!("{space_char}");
                        i += 1;
                    }
                    if !top {
                        print!("{label}");
                        i += label_len;
                    }
                    if i < end_time {
                        while i + 1 < end_time.min(next_t) {
                            print!("{space_char}");
                            i += 1;
                        }
                        if i + 1 == next_t {
                            if top {
                                print!(" ");
                            } else if label_trimmed == "" {
                                print!("/");
                            } else if label_trimmed == "1" {
                                print!("\\");
                            } else {
                                print!("X");
                            }
                            i += 1;
                        } else {
                            print!("{space_char}");
                            i += 1;
                        }
                    }
                } else {
                    while i < end_time {
                        print!("_");
                        i += 1;
                    }
                }
            }
            println!();
        };
        draw_line(true);
        draw_line(false);
    }
}
