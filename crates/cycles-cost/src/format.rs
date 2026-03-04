use crate::costs::Row;

pub fn print_table(
    title: &str,
    rows: &[Row],
    subnet_sizes: &[usize],
    fmt_val: impl Fn(u128) -> String,
) {
    let col_width = 22;
    let name_width = 40;
    let total_width = name_width + (1 + col_width) * subnet_sizes.len();

    println!("{title}");
    println!("{}", "=".repeat(total_width));
    println!();
    print!("{:<name_width$}", "Transaction");
    for &n in subnet_sizes {
        print!(" {:>col_width$}", format!("{n}-node app subnet"));
    }
    println!();
    println!("{}", "-".repeat(total_width));

    for entry in rows {
        match entry {
            Row::Separator => println!(),
            Row::Data { name, values } => {
                print!("{:<name_width$}", name);
                for v in values {
                    print!(" {:>col_width$}", fmt_val(*v));
                }
                println!();
            }
        }
    }
}

pub fn fmt_cycles(v: u128) -> String {
    let s = v.to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push('_');
        }
        result.push(ch);
    }
    result
}

pub fn fmt_usd(usd: f64) -> String {
    if usd >= 1.0 {
        format!("${:.4}", usd)
    } else if usd >= 0.01 {
        format!("${:.6}", usd)
    } else if usd >= 0.0001 {
        format!("${:.8}", usd)
    } else {
        format!("${:.12}", usd)
    }
}

pub fn fmt_icp(icp: f64) -> String {
    if icp >= 1.0 {
        format!("{:.4} ICP", icp)
    } else if icp >= 0.01 {
        format!("{:.6} ICP", icp)
    } else if icp >= 0.0001 {
        format!("{:.8} ICP", icp)
    } else {
        format!("{:.12} ICP", icp)
    }
}
