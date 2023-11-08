#![allow(dead_code)]

mod fstapi;

fn main() {
    let mut fst = fstapi::Reader::open("test.fst").expect("open fst file");
    println!(
        "version: {}, date: {}",
        fst.version_string(),
        fst.date_string().trim()
    );
    println!(
        "start: {}, end: {}, timescale: 10^{}, timezero: {}",
        fst.start_time(),
        fst.end_time(),
        fst.timescale(),
        fst.timezero(),
    );
    println!(
        "value_change_section_count: {}, current_scope_len: {}, max_handle: {}",
        fst.value_change_section_count(),
        fst.current_scope_len(),
        fst.max_handle(),
    );

    println!("\nhier:");
    let mut vars: std::collections::HashMap<
        fstapi::fstHandle,
        (Vec<String>, fstapi::VarType, usize, Vec<(u64, String)>),
    > = std::collections::HashMap::new();
    let mut aliases: Vec<(Vec<String>, fstapi::fstHandle)> = vec![];
    let mut scope: Vec<String> = vec![];
    let to_show = vec!["test".to_string()];
    let mut i = 0;
    while let Some(hier) = fst.next_hier() {
        match &hier {
            fstapi::Hier::Scope(s) => scope.push(s.name().to_string()),
            fstapi::Hier::Upscope => {
                scope.pop();
            }
            fstapi::Hier::Var(v)
                if scope == to_show && v.typ() != fstapi::VarType::VcdParameter =>
            {
                let mut name = scope.clone();
                name.push(v.name().to_string());
                if v.is_alias() {
                    aliases.push((name, v.handle()));
                } else {
                    vars.insert(v.handle(), (name, v.typ(), v.length(), vec![]));
                }
            }
            _ => {}
        }
        println!("{i}: {:?}", hier);
        i += 1;
    }

    println!("\nblocks:");
    let end_time = 199.min(fst.end_time());
    let limit_keys: Vec<_> = vars.keys().copied().collect();
    let mut crit_time = vec![];
    fst.foreach_block(
        Some(fst.start_time()..end_time),
        Some(&limit_keys),
        |time, handle, val| {
            println!("block: {time} {handle} {val:?}");
            let v = vars.get_mut(&handle).unwrap();
            crit_time.push(time);
            v.3.push((time, val.to_string()));
        },
    );
    crit_time.sort();
    crit_time.dedup();

    println!("\nvars:");
    {
        println!("$time");
        let mut i = fst.start_time();
        for t in crit_time {
            if t < i || t >= end_time {
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
    for (h, v) in &vars {
        println!("{h}: {} {:?} {}", v.0[1], v.1, v.2);
        let draw_line = |top: bool| {
            let mut vals = v.3.iter().peekable();
            let mut i = fst.start_time();
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
