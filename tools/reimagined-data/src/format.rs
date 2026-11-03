//! Port of the d2r-reimagined.com template renderer (src/utilities/format-template.ts and the
//! `line()` helper in src/lib/i18n.ts), so tooltips read exactly like the website's.

use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Arg {
    Num(f64),
    Str(String),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Tok {
    Percent,
    Indexed(usize),
    Color,
    Num(char), // 'd' 'D' 'i' or '+' for %+d
    Str,
}

/// Split a template into literal text and tokens.
fn tokenize(t: &str) -> Vec<Result<String, Tok>> {
    let mut out: Vec<Result<String, Tok>> = Vec::new();
    let chars: Vec<char> = t.chars().collect();
    let mut lit = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' && i + 1 < chars.len() {
            let n = chars[i + 1];
            let tok = match n {
                '%' => Some((Tok::Percent, 2)),
                '+' if i + 2 < chars.len() && chars[i + 2] == 'd' => Some((Tok::Num('+'), 3)),
                'd' | 'D' | 'i' => Some((Tok::Num(n), 2)),
                's' | 'S' => Some((Tok::Str, 2)),
                'c' if i + 2 < chars.len() && chars[i + 2].is_ascii_digit() => Some((Tok::Color, 3)),
                c if c.is_ascii_digit() => Some((Tok::Indexed(c as usize - '0' as usize), 2)),
                _ => None,
            };
            if let Some((tok, len)) = tok {
                if !lit.is_empty() {
                    out.push(Ok(std::mem::take(&mut lit)));
                }
                out.push(Err(tok));
                i += len;
                continue;
            }
        }
        lit.push(chars[i]);
        i += 1;
    }
    if !lit.is_empty() {
        out.push(Ok(lit));
    }
    out
}

fn decimal(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.6}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn fmt_arg(a: Option<&Arg>, tok: Tok) -> String {
    match (a, tok) {
        (None, _) => String::new(),
        (Some(Arg::Num(n)), Tok::Num('+')) => format!("{}{}", if *n >= 0.0 { "+" } else { "" }, decimal(*n)),
        (Some(Arg::Num(n)), _) => decimal(*n),
        (Some(Arg::Str(s)), _) => s.clone(),
    }
}

fn is_num(a: Option<&Arg>) -> Option<f64> {
    match a {
        Some(Arg::Num(n)) if n.is_finite() => Some(*n),
        _ => None,
    }
}

pub fn count_numeric_slots(template: &str, string_args: usize) -> usize {
    let toks: Vec<Tok> = tokenize(template).into_iter().filter_map(|t| t.err()).collect();
    let indexed: Vec<usize> = toks.iter().filter_map(|t| if let Tok::Indexed(i) = t { Some(*i) } else { None }).collect();
    if !indexed.is_empty() {
        let slots = indexed.iter().map(|i| i + 1).max().unwrap_or(0);
        return slots.saturating_sub(string_args);
    }
    toks.iter().filter(|t| matches!(t, Tok::Num(_))).count()
}

pub fn format_template(template: &str, args: &[Arg]) -> String {
    if template.is_empty() {
        return String::new();
    }
    let parts = tokenize(template);
    let toks: Vec<Tok> = parts.iter().filter_map(|p| p.as_ref().err().copied()).collect();
    let sequential: Vec<Tok> = toks.iter().copied().filter(|t| matches!(t, Tok::Num(_) | Tok::Str)).collect();
    let numeric_count = sequential.iter().filter(|t| matches!(t, Tok::Num(_))).count();
    let mut pair_budget = (args.len() as isize - sequential.len() as isize).max(0).min(numeric_count as isize) as usize;

    let indexed_slots = toks.iter().filter_map(|t| if let Tok::Indexed(i) = t { Some(i + 1) } else { None }).max().unwrap_or(0);
    let resolved_indexed: Option<Vec<String>> = if indexed_slots > 0 && args.len() > indexed_slots {
        let mut res = Vec::new();
        let mut p = 0;
        let mut budget = args.len() - indexed_slots;
        for _ in 0..indexed_slots {
            match (is_num(args.get(p)), is_num(args.get(p + 1))) {
                (Some(a), Some(b)) if budget > 0 => {
                    res.push(if a == b { decimal(a) } else { format!("{}-{}", decimal(a), decimal(b)) });
                    p += 2;
                    budget -= 1;
                }
                _ => {
                    res.push(fmt_arg(args.get(p), Tok::Indexed(0)));
                    p += 1;
                }
            }
        }
        Some(res)
    } else {
        None
    };

    let mut out = String::new();
    let mut seq = 0usize;
    for part in parts {
        match part {
            Ok(text) => out.push_str(&text),
            Err(Tok::Percent) => out.push('%'),
            Err(Tok::Color) => {}
            Err(Tok::Indexed(i)) => match &resolved_indexed {
                Some(r) => out.push_str(r.get(i).map(|s| s.as_str()).unwrap_or("")),
                None => out.push_str(&fmt_arg(args.get(i), Tok::Indexed(i))),
            },
            Err(tok @ Tok::Num(_)) if pair_budget > 0 => {
                pair_budget -= 1;
                let min = args.get(seq);
                let max = args.get(seq + 1);
                seq += 2;
                let min_s = fmt_arg(min, tok);
                match (is_num(min), is_num(max)) {
                    (Some(a), Some(b)) if a != b => {
                        let max_tok = if tok == Tok::Num('+') { Tok::Num('d') } else { tok };
                        out.push_str(&format!("{}-{}", min_s, fmt_arg(max, max_tok)));
                    }
                    _ => out.push_str(&min_s),
                }
            }
            Err(tok) => {
                out.push_str(&fmt_arg(args.get(seq), tok));
                seq += 1;
            }
        }
    }
    out
}

/// The English strings: the game's exported strings plus the website's own UI strings.
pub struct Strings {
    map: HashMap<String, String>,
}

/// "[ms]Le %s[fs]La %s" -> "Le %s" (first non-empty gender variant)
fn strip_gender(v: &str) -> String {
    if !v.starts_with('[') {
        return v.to_string();
    }
    let mut parts = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = v.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '[' && i + 3 < chars.len() && chars[i + 1].is_ascii_lowercase() && chars[i + 2].is_ascii_lowercase() && chars[i + 3] == ']' {
            parts.push(std::mem::take(&mut cur));
            i += 4;
            continue;
        }
        cur.push(chars[i]);
        i += 1;
    }
    parts.push(cur);
    parts.into_iter().find(|p| !p.is_empty()).unwrap_or_default()
}

impl Strings {
    pub fn load(game_strings: &str, ui_strings_ts: &str) -> Strings {
        let mut map: HashMap<String, String> = HashMap::new();
        // ui-strings.ts: take the 'enUS': { 'key': 'value', ... } block
        if let Some(start) = ui_strings_ts.find("'enUS': {") {
            for line in ui_strings_ts[start + 9..].lines() {
                let l = line.trim();
                if l.starts_with('}') {
                    break;
                }
                if let Some((k, v)) = parse_ts_pair(l) {
                    map.insert(k, v);
                }
            }
        }
        if let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(game_strings) {
            for (k, v) in obj {
                if let Value::String(s) = v {
                    map.entry(k).or_insert_with(|| strip_gender(&s));
                }
            }
        }
        Strings { map }
    }

    pub fn get(&self, key: &str) -> String {
        if key.is_empty() {
            return String::new();
        }
        self.map.get(key).cloned().unwrap_or_else(|| key.to_string())
    }

    fn args(&self, v: Option<&Value>) -> Vec<Arg> {
        match v {
            Some(Value::Array(a)) => a
                .iter()
                .map(|x| match x {
                    Value::Number(n) => Arg::Num(n.as_f64().unwrap_or(0.0)),
                    Value::String(s) => Arg::Str(self.get(s)),
                    Value::Bool(b) => Arg::Str(b.to_string()),
                    _ => Arg::Str(String::new()),
                })
                .collect(),
            _ => vec![],
        }
    }

    /// `t(key, args)` from the website
    pub fn t(&self, key: &str, args: &[Arg]) -> String {
        format_template(&self.get(key), args)
    }

    /// A keyed property line `{ key, args, perLevel, classOnly, itemsRequired, fullSet, ... }`.
    pub fn line(&self, v: &Value) -> String {
        let key = v.get("key").and_then(|k| k.as_str()).unwrap_or("");
        let mut args = self.args(v.get("args"));
        let mut template = self.get(key);
        // the website's two special cases
        if key == "healperhit" && !args.is_empty() {
            template = format!("%d {}", template);
        }
        if key == "strSkillRandomFromSkillClass" && args.len() >= 2 {
            if let (Arg::Num(a), Arg::Num(b)) = (&args[0], &args[1]) {
                let n = if a.trunc() == b.trunc() { format!("{}", a.trunc()) } else { format!("{}-{}", a.trunc(), b.trunc()) };
                let rest: Vec<Arg> = args[2..].to_vec();
                if rest.len() < 2 {
                    template = template.trim_end().trim_end_matches("%s").trim_end().to_string();
                }
                args = std::iter::once(Arg::Str(n)).chain(rest).collect();
            }
        }
        let mut s = format_template(&template, &args);
        if v.get("perLevel").and_then(|x| x.as_bool()).unwrap_or(false) {
            s += &self.get("strPerCharacterLevelSuffix");
        }
        if let Some(q) = v.get("qualifier").and_then(|x| x.as_str()) {
            let k = match q {
                "weapon" => "strRuneScopeWeapon",
                "shield" => "strRuneScopeShield",
                "armor" => "strRuneScopeArmor",
                _ => "",
            };
            if !k.is_empty() {
                s += &self.get(k);
            }
        }
        if let Some(c) = v.get("classOnly").and_then(|x| x.as_str()) {
            if !c.is_empty() {
                s = format!("{} {}", s, self.get(c));
            }
        }
        if let Some(n) = v.get("itemsRequired").and_then(|x| x.as_f64()) {
            s = format_template(&self.get("strPartialSetBonus"), &[Arg::Str(s), Arg::Num(n)]);
        } else if v.get("fullSet").and_then(|x| x.as_bool()).unwrap_or(false) {
            s = format_template(&self.get("strFullSetBonus"), &[Arg::Str(s)]);
        }
        s
    }
}

/// Parse one `'key': 'value',` line of a TypeScript object literal.
fn parse_ts_pair(l: &str) -> Option<(String, String)> {
    let (k, rest) = parse_quoted(l)?;
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let (v, _) = parse_quoted(rest)?;
    Some((k, v))
}

fn parse_quoted(s: &str) -> Option<(String, &str)> {
    let q = s.chars().next()?;
    if q != '\'' && q != '"' {
        return None;
    }
    let mut out = String::new();
    let mut esc = false;
    for (i, c) in s.char_indices().skip(1) {
        if esc {
            out.push(match c {
                'n' => '\n',
                other => other,
            });
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else if c == q {
            return Some((out, &s[i + 1..]));
        } else {
            out.push(c);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ranges() {
        assert_eq!(format_template("%+d Defense", &[Arg::Num(180.0), Arg::Num(190.0)]), "+180-190 Defense");
        assert_eq!(format_template("-%d%% to Enemy Cold Resistance", &[Arg::Num(15.0), Arg::Num(25.0)]), "-15-25% to Enemy Cold Resistance");
        assert_eq!(format_template("Level %d %s Aura When Equipped", &[Arg::Num(5.0), Arg::Str("Resist Cold".into())]), "Level 5 Resist Cold Aura When Equipped");
        assert_eq!(format_template("%+d to Elemental Skills", &[Arg::Num(3.0), Arg::Num(4.0)]), "+3-4 to Elemental Skills");
        assert_eq!(format_template("Defense: %d-%d", &[Arg::Num(283.0), Arg::Num(345.0)]), "Defense: 283-345");
        assert_eq!(format_template("%0%% Reanimate as: %1", &[Arg::Num(5.0), Arg::Num(10.0), Arg::Str("Voltshade".into())]), "5-10% Reanimate as: Voltshade");
        assert_eq!(strip_gender("[ms]Le %s[fs]La %s"), "Le %s");
        assert_eq!(parse_ts_pair("'label_rarity': 'Rarity: %s',"), Some(("label_rarity".into(), "Rarity: %s".into())));
    }
}
