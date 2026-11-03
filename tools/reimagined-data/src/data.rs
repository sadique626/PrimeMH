//! Loads the website's data files and turns each entry into a tooltip (a list of coloured
//! lines), following src/lib/components/CatalogCard.svelte.

use serde_json::Value;
use std::path::Path;

use crate::format::{Arg, Strings};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Uniques,
    Sets,
    Runewords,
    Bases,
    Affixes,
    Cube,
    Orbs,
}

impl Tab {
    pub const ALL: [Tab; 7] = [Tab::Uniques, Tab::Sets, Tab::Runewords, Tab::Bases, Tab::Affixes, Tab::Cube, Tab::Orbs];
    pub fn label(self) -> &'static str {
        match self {
            Tab::Uniques => "Uniques",
            Tab::Sets => "Sets",
            Tab::Runewords => "Runewords",
            Tab::Bases => "Bases",
            Tab::Affixes => "Affixes",
            Tab::Cube => "Cube",
            Tab::Orbs => "Orbs",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Unique,
    Set,
    Rarity,
    Base,
    Requirement,
    Magic,
    Plain,
    Heading,
    Note,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub text: String,
    pub kind: Kind,
    pub indent: bool,
}

#[derive(Clone, Debug)]
pub struct Badge {
    pub text: String,
    pub red: bool,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub tab: Tab,
    pub title: String,
    pub title_kind: Kind,
    pub subtitle: String,
    pub badges: Vec<Badge>,
    pub lines: Vec<Line>,
    /// lowercase title + every line, for searching
    pub search: String,
}

struct Builder<'a> {
    s: &'a Strings,
    lines: Vec<Line>,
}

impl<'a> Builder<'a> {
    fn push(&mut self, text: String, kind: Kind) {
        if !text.trim().is_empty() {
            self.lines.push(Line { text, kind, indent: false });
        }
    }
    fn keyed(&mut self, lines: Option<&Value>, kind: Kind) {
        if let Some(Value::Array(a)) = lines {
            for l in a {
                self.push(self.s.line(l), kind);
                // property groups: "one of" children
                if let Some(Value::Array(children)) = l.get("children") {
                    for c in children {
                        let t = self.s.line(c);
                        if !t.trim().is_empty() {
                            self.lines.push(Line { text: t, kind, indent: true });
                        }
                    }
                }
            }
        }
    }
    fn keyed_filtered(&mut self, lines: Option<&Value>, keys: &[&str], include: bool, kind: Kind) {
        if let Some(Value::Array(a)) = lines {
            let picked: Vec<Value> = a
                .iter()
                .filter(|l| keys.contains(&l.get("key").and_then(|k| k.as_str()).unwrap_or("")) == include)
                .cloned()
                .collect();
            self.keyed(Some(&Value::Array(picked)), kind);
        }
    }
}

const EQUIPMENT_EARLY: [&str; 9] = [
    "strDefense", "strDefenseRange", "strDefenseRangeRange", "strChanceToBlock", "strSmiteDamage",
    "strKickDamage", "strWeaponDamageGenericRange", "strWeaponDamageOneHand", "strWeaponDamageTwoHand",
];
const REQUIREMENTS: [&str; 6] = [
    "strRequiredClass", "strRequiredDexterity", "strRequiredStrength", "strRequiredDexterityRange",
    "strRequiredStrengthRange", "strRequiredLevelRange",
];

fn s<'v>(v: &'v Value, k: &str) -> &'v str {
    v.get(k).and_then(|x| x.as_str()).unwrap_or("")
}
fn n(v: &Value, k: &str) -> Option<f64> {
    v.get(k).and_then(|x| x.as_f64())
}
fn has_level_range(lines: Option<&Value>) -> bool {
    matches!(lines, Some(Value::Array(a)) if a.iter().any(|l| s(l, "key") == "strRequiredLevelRange"))
}

fn common_badges(v: &Value, item_level: bool) -> Vec<Badge> {
    let mut b = Vec::new();
    match s(v, "Vanilla") {
        "Y" => b.push(Badge { text: "Vanilla".into(), red: false }),
        "N" => b.push(Badge { text: "Reimagined".into(), red: true }),
        _ => {}
    }
    if item_level {
        if let Some(il) = n(v, "ItemLevel") {
            b.push(Badge { text: format!("Item Level {}", il), red: false });
            if il >= 100.0 {
                b.push(Badge { text: "Drop Only".into(), red: false });
            }
        }
    } else if let Some(rl) = n(v, "RequiredLevel") {
        b.push(Badge { text: format!("Level {}", rl), red: false });
    }
    b
}

/// The equipment part of a unique / set item: base name, defense or damage, requirements.
fn equipment(b: &mut Builder, item: &Value) {
    let eq = item.get("Equipment");
    if let Some(eq) = eq {
        b.push(b.s.get(s(eq, "NameKey")), Kind::Base);
        b.keyed_filtered(eq.get("Lines"), &EQUIPMENT_EARLY, true, Kind::Base);
        if let Some(Value::Array(dts)) = eq.get("DamageTypes") {
            for d in dts {
                b.keyed(d.get("Lines"), Kind::Base);
            }
        }
        b.keyed_filtered(eq.get("Lines"), &REQUIREMENTS, true, Kind::Requirement);
    }
    if !has_level_range(eq.and_then(|e| e.get("Lines"))) {
        let rl = n(item, "RequiredLevel").unwrap_or(1.0);
        b.push(b.s.t("strRequiredLevel", &[Arg::Num(rl)]), Kind::Requirement);
    }
}

fn finish(tab: Tab, title: String, title_kind: Kind, subtitle: String, badges: Vec<Badge>, lines: Vec<Line>) -> Entry {
    let mut search = title.to_lowercase();
    for l in &lines {
        search.push('\n');
        search.push_str(&l.text.to_lowercase());
    }
    Entry { tab, title, title_kind, subtitle, badges, lines, search }
}

fn rarity(st: &Strings, v: &Value) -> String {
    match n(v, "Rarity") {
        Some(r) if r > 0.0 => st.t("label_rarity", &[Arg::Num(r)]),
        _ => String::new(),
    }
}

fn read(dir: &Path, name: &str) -> Vec<Value> {
    std::fs::read_to_string(dir.join("keyed").join(name))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| match v {
            Value::Array(a) => Some(a),
            _ => None,
        })
        .unwrap_or_default()
}

/// Rows of magicprefix.txt / magicsuffix.txt as (name, level). A cube recipe's "pre=N" /
/// "suf=N" is the Nth data row (0-based, header excluded).
fn affix_rows(dir: &Path, file: &str) -> Vec<(String, f64, String)> {
    let Ok(text) = std::fs::read(dir.join(file)) else { return vec![] };
    let text: String = text.iter().map(|b| *b as char).collect(); // latin-1
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap_or("").split('\t').collect();
    let level_col = header.iter().position(|h| h.eq_ignore_ascii_case("level")).unwrap_or(5);
    let desc_col = header.iter().position(|h| h.contains("Description")).unwrap_or(1);
    lines
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            (
                c.first().unwrap_or(&"").to_string(),
                c.get(level_col).and_then(|v| v.trim().parse().ok()).unwrap_or(-1.0),
                c.get(desc_col).unwrap_or(&"").to_string(),
            )
        })
        .collect()
}

/// "Force Prefix (365)" -> "Force Prefix: Coral (+15-20% Lightning Resist)"
struct AffixNames {
    prefixes: Vec<(String, f64, String)>,
    suffixes: Vec<(String, f64, String)>,
    /// (json file, name, level) -> rendered stat lines
    lines: std::collections::HashMap<(bool, String, i64), String>,
}

impl AffixNames {
    /// Short stat name of a forced affix, e.g. "Lightning Resist +#%" -> "Lightning Resist"
    fn short(&self, q: &Value) -> Option<String> {
        let prefix = match s(q, "key") {
            "strCubeQualifierForcePrefixN" => true,
            "strCubeQualifierForceSuffixN" => false,
            _ => return None,
        };
        let n = q.get("args").and_then(|a| a.get(0)).and_then(|a| a.as_f64())? as usize;
        let (name, _, desc) = if prefix { self.prefixes.get(n)? } else { self.suffixes.get(n)? };
        let cleaned: String = desc
            .split_whitespace()
            .filter(|w| !w.contains('#') && !w.chars().all(|c| c.is_ascii_digit() || "+-%".contains(c)))
            .collect::<Vec<_>>()
            .join(" ");
        let cleaned = cleaned.trim_matches(|c: char| c == ':' || c == '+' || c == '-' || c == ' ').to_string();
        Some(if cleaned.is_empty() { name.clone() } else { cleaned })
    }

    fn describe(&self, st: &Strings, q: &Value) -> Option<String> {
        let key = s(q, "key");
        let prefix = match key {
            "strCubeQualifierForcePrefixN" => true,
            "strCubeQualifierForceSuffixN" => false,
            _ => return None,
        };
        let n = q.get("args").and_then(|a| a.get(0)).and_then(|a| a.as_f64())? as usize;
        let (name, level, _) = if prefix { self.prefixes.get(n)? } else { self.suffixes.get(n)? };
        let label = if prefix { "Force Prefix" } else { "Force Suffix" };
        let stats = self.lines.get(&(prefix, name.clone(), *level as i64)).cloned().unwrap_or_default();
        let shown = st.get(name);
        Some(if stats.is_empty() { format!("{}: {}", label, shown) } else { format!("{}: {} ({})", label, shown, stats) })
    }
}

pub fn load(dir: &Path) -> Result<(Strings, Vec<Entry>), String> {
    let game = std::fs::read_to_string(dir.join("strings").join("enUS.json"))
        .map_err(|e| format!("could not read {}: {}", dir.join("strings").join("enUS.json").display(), e))?;
    let ui = std::fs::read_to_string(dir.join("ui-strings.ts")).unwrap_or_default();
    let st = Strings::load(&game, &ui);
    let mut out: Vec<Entry> = Vec::new();

    let mut affixes = AffixNames {
        prefixes: affix_rows(dir, "magicprefix.txt"),
        suffixes: affix_rows(dir, "magicsuffix.txt"),
        lines: std::collections::HashMap::new(),
    };
    for (prefix, file) in [(true, "magicprefix.json"), (false, "magicsuffix.json")] {
        for a in read(dir, file) {
            let text: Vec<String> = match a.get("Lines") {
                Some(Value::Array(l)) => l.iter().map(|x| st.line(x)).filter(|t| !t.trim().is_empty()).collect(),
                _ => vec![],
            };
            let lvl = n(&a, "Level").unwrap_or(-1.0) as i64;
            affixes.lines.insert((prefix, s(&a, "NameKey").to_string(), lvl), text.join(", "));
        }
    }

    // uniques
    for u in read(dir, "uniques.json") {
        if u.get("Enabled").and_then(|e| e.as_bool()) == Some(false) {
            continue;
        }
        let mut b = Builder { s: &st, lines: vec![] };
        equipment(&mut b, &u);
        b.keyed(u.get("Lines"), Kind::Magic);
        let title = st.get(s(&u, "Index"));
        out.push(finish(Tab::Uniques, title, Kind::Unique, rarity(&st, &u), common_badges(&u, true), b.lines));
    }

    // sets: one entry per set, bonuses first, then each piece
    for set in read(dir, "sets.json") {
        let mut b = Builder { s: &st, lines: vec![] };
        b.keyed(set.get("PartialBonuses"), Kind::Set);
        b.keyed(set.get("FullBonuses"), Kind::Set);
        if let Some(Value::Array(items)) = set.get("SetItems") {
            for it in items {
                b.lines.push(Line { text: st.get(s(it, "Index")), kind: Kind::Heading, indent: false });
                let r = rarity(&st, it);
                b.push(r, Kind::Rarity);
                equipment(&mut b, it);
                b.keyed(it.get("Lines"), Kind::Magic);
                if let Some(Value::Array(bonuses)) = it.get("SetBonuses") {
                    for bonus in bonuses {
                        b.keyed(Some(bonus), Kind::Set);
                    }
                }
            }
        }
        let title = st.get(s(&set, "Index"));
        out.push(finish(Tab::Sets, title, Kind::Set, String::new(), common_badges(&set, false), b.lines));
    }

    // runewords
    for rw in read(dir, "runewords.json") {
        if rw.get("Enabled").and_then(|e| e.as_bool()) == Some(false) {
            continue;
        }
        let mut b = Builder { s: &st, lines: vec![] };
        let types: Vec<String> = match rw.get("Types") {
            Some(Value::Array(t)) => t.iter().map(|x| st.get(if x.is_string() { x.as_str().unwrap() } else { s(x, "Index") })).collect(),
            _ => vec![],
        };
        b.push(types.join(&st.get("label_or")), Kind::Base);
        let runes: Vec<String> = match rw.get("Runes") {
            Some(Value::Array(r)) => r.iter().map(|x| st.get(s(x, "NameKey"))).collect(),
            _ => vec![],
        };
        b.push(runes.join(" + "), Kind::Plain);
        if !has_level_range(rw.get("Lines")) {
            b.push(st.t("strRequiredLevel", &[Arg::Num(n(&rw, "RequiredLevel").unwrap_or(1.0))]), Kind::Requirement);
        }
        b.keyed(rw.get("Lines"), Kind::Magic);
        let title = st.get(s(&rw, "Index"));
        out.push(finish(Tab::Runewords, title, Kind::Unique, rarity(&st, &rw), common_badges(&rw, false), b.lines));
    }

    // bases
    for file in ["weapons.json", "armors.json"] {
        for base in read(dir, file) {
            let mut b = Builder { s: &st, lines: vec![] };
            let ty = base.get("Type").map(|t| if t.is_string() { t.as_str().unwrap().to_string() } else { s(t, "Index").to_string() }).unwrap_or_default();
            b.push(st.get(&ty), Kind::Base);
            if let Some(Value::Array(dts)) = base.get("DamageTypes") {
                for d in dts {
                    b.keyed(d.get("Lines"), Kind::Base);
                }
            }
            b.keyed_filtered(base.get("Lines"), &REQUIREMENTS, false, Kind::Base);
            b.keyed_filtered(base.get("Lines"), &REQUIREMENTS, true, Kind::Requirement);
            if let Some(rl) = n(&base, "RequiredLevel") {
                b.push(st.t("strRequiredLevel", &[Arg::Num(rl)]), Kind::Requirement);
            }
            if let Some(gs) = base.get("GemSockets").and_then(|g| g.as_str()) {
                b.push(format!("Sockets: {}", gs), Kind::Note);
            }
            let title = st.get(s(&base, "NameKey"));
            out.push(finish(Tab::Bases, title, Kind::Plain, String::new(), common_badges(&base, true), b.lines));
        }
    }

    // affixes
    for file in ["magicprefix.json", "magicsuffix.json"] {
        for a in read(dir, file) {
            let mut b = Builder { s: &st, lines: vec![] };
            let max = n(&a, "MaxLevel").unwrap_or(0.0);
            let lvl = n(&a, "Level").unwrap_or(0.0);
            b.push(
                format!("{} · Level {}{}", s(&a, "PType"), lvl, if max > 0.0 { format!("–{}", max) } else { "+".into() }),
                Kind::Base,
            );
            if !s(&a, "ClassSpecific").is_empty() {
                b.push(s(&a, "ClassSpecific").to_string(), Kind::Requirement);
            }
            let types: Vec<String> = match a.get("Types") {
                Some(Value::Array(t)) => t.iter().map(|x| st.get(x.as_str().unwrap_or(""))).collect(),
                _ => vec![],
            };
            b.push(types.join(", "), Kind::Note);
            b.keyed(a.get("Lines"), Kind::Magic);
            let title = st.get(s(&a, "NameKey"));
            out.push(finish(Tab::Affixes, title, Kind::Magic, String::new(), vec![], b.lines));
        }
    }

    // cube recipes and orbs
    for r in read(dir, "cube-recipes.json") {
        if r.get("Enabled").and_then(|e| e.as_bool()) == Some(false) {
            continue;
        }
        let is_orb = matches!(r.get("Notes"), Some(Value::Array(ns)) if ns.iter().any(|x| s(x, "key").starts_with("strCubeNoteOrb")));
        let mut b = Builder { s: &st, lines: vec![] };
        let qualifiers = |v: &Value| -> String {
            match v.get("Qualifiers") {
                Some(Value::Array(q)) => q
                    .iter()
                    .filter(|x| s(x, "key") != "strCubeQualifierQuantityN")
                    .map(|x| format!(" [{}]", affixes.describe(&st, x).unwrap_or_else(|| st.line(x))))
                    .collect(),
                _ => String::new(),
            }
        };
        let qty = |v: &Value| -> String {
            match n(v, "Quantity") {
                Some(q) if q > 1.0 => format!("{} × ", q),
                _ => String::new(),
            }
        };
        b.lines.push(Line { text: "Inputs".into(), kind: Kind::Heading, indent: false });
        if let Some(Value::Array(inputs)) = r.get("Inputs") {
            for i in inputs {
                let name = i.get("Name").map(|x| st.line(x)).unwrap_or_default();
                b.push(format!("{}{}{}", qty(i), name, qualifiers(i)), Kind::Plain);
            }
        }
        b.lines.push(Line { text: "Outputs".into(), kind: Kind::Heading, indent: false });
        if let Some(Value::Object(outs)) = r.get("Outputs") {
            for (_, o) in outs {
                let name = o.get("Name").map(|x| st.line(x)).unwrap_or_default();
                b.push(format!("{}{}{}", qty(o), name, qualifiers(o)), Kind::Plain);
                b.keyed(o.get("Lines"), Kind::Magic);
            }
        }
        b.keyed(r.get("Notes"), Kind::Note);
        b.keyed(r.get("Details"), Kind::Note);
        let desc = s(&r, "Description");
        let inputs_title = match r.get("Inputs") {
            Some(Value::Array(i)) => i
                .iter()
                .map(|x| format!("{}{}", qty(x), x.get("Name").map(|n| st.line(n)).unwrap_or_default()))
                .collect::<Vec<_>>()
                .join(" + "),
            _ => "Recipe".into(),
        };
        let title = if !desc.is_empty() {
            st.get(desc)
        } else {
            recipe_name(&r, &st, &affixes).unwrap_or(inputs_title)
        };
        out.push(finish(if is_orb { Tab::Orbs } else { Tab::Cube }, title, Kind::Plain, String::new(), vec![], b.lines));
    }

    Ok((st, out))
}

/// A short "what it does" name for a cube recipe, e.g. "Lightning Resist Magic Ring",
/// "Corrupt Amulet: +1 to All Skills", "Upgrade Rare Jewel -> Unique Jewel".
fn recipe_name(r: &Value, st: &Strings, affixes: &AffixNames) -> Option<String> {
    let quality = |q: &str| -> Option<&'static str> {
        Some(match q {
            "strCubeQualifierMagicItem" => "Magic",
            "strCubeQualifierRareItem" => "Rare",
            "strCubeQualifierUniqueItem" => "Unique",
            "strCubeQualifierSetItem" => "Set",
            "strCubeQualifierCraftedItem" => "Crafted",
            "strCubeQualifierNormalQuality" | "strCubeQualifierNormalItem" | "strCubeQualifierBasicItem" => "Normal",
            "strCubeQualifierHighQualitySuperior" | "strCubeQualifierHighQualityItemSuperior" => "Superior",
            "strCubeQualifierLowQuality" => "Low Quality",
            "strCubeQualifierEthereal" => "Ethereal",
            "strCubeQualifierExceptionalItem" => "Exceptional",
            "strCubeQualifierEliteItem" => "Elite",
            _ => return None,
        })
    };
    let quals = |v: &Value| -> Vec<&'static str> {
        match v.get("Qualifiers") {
            Some(Value::Array(q)) => q.iter().filter_map(|x| quality(s(x, "key"))).collect(),
            _ => vec![],
        }
    };
    let verb = |note: &str| -> Option<&'static str> {
        Some(match note {
            "strCubeNoteOrbOfCorruption" => "Corrupt",
            "strCubeNoteItemCrafting" => "Craft",
            "strCubeNoteItemEnchantment" => "Enchant",
            "strCubeNoteSocketPunch" | "strCubeNoteSocketItem" | "strCubeNoteOrbOfSocketing" => "Socket",
            "strCubeNoteRerollItem" | "strCubeNoteRerollUberAncient" => "Reroll",
            "strCubeNoteRecycle" => "Recycle",
            "strCubeNoteOrbOfConversion" | "strCubeNoteKeyConversion" => "Convert",
            "strCubeNoteOrbOfInfusion" => "Infuse",
            "strCubeNoteSunderItemCrafting" => "Sunder",
            "strCubeNoteOrbOfAssemblage" => "Assemble",
            "strCubeNoteTristramUberSouls" => "Uber Soul",
            "strCubeNoteUberCharmUpgrade" | "strCubeNoteSplashCharmUpgrade" | "strCubeNoteBaseTierUpgrade" | "strCubeNoteJewelUpgrade" => "Upgrade",
            "strCubeNoteForceWhite" => "Make Normal",
            "strCubeNoteForceEthereal" => "Make Ethereal",
            "strCubeNoteOrbOfShadows" => "Shadow",
            "strCubeNoteRepairEthereal" | "strCubeNoteRepairNonEthereal" => "Repair",
            "strCubeNotePliers" => "Unsocket",
            "strCubeNoteReplenishQuiver" => "Refill",
            _ => return None,
        })
    };
    // names picked by hand for recipes the rules can't describe well
    const OVERRIDES: [(&str, &str); 1] = [("cr-7547ae5e57f884d2", "Warp Ring")];
    if let Some((_, n)) = OVERRIDES.iter().find(|(id, _)| *id == s(r, "Id")) {
        return Some(n.to_string());
    }
    // the real result: skip the "gem bag is given back" output that many recipes also have
    let outputs: Vec<&Value> = r.get("Outputs").and_then(|o| o.as_object()).map(|o| o.values().collect()).unwrap_or_default();
    let output = outputs
        .iter()
        .rev()
        .find(|o| o.get("Name").map(|n| s(n, "key")) != Some("strCubeOutputReturnGemBag"))
        .or(outputs.first())
        .copied()?;
    let out_key = output.get("Name").map(|n| s(n, "key")).unwrap_or("");
    let notes: Vec<&str> = match r.get("Notes") {
        Some(Value::Array(n)) => n.iter().map(|x| s(x, "key")).collect(),
        _ => vec![],
    };
    let action = notes.iter().find_map(|n| verb(n));
    // the item being worked on: the first input that isn't the gem bag paying for it
    let input = match r.get("Inputs") {
        Some(Value::Array(i)) => i.iter().find(|x| x.get("Name").map(|n| s(n, "key")) != Some("bag")).or(i.first()),
        _ => None,
    };
    let join = |parts: Vec<String>| parts.into_iter().filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ");
    let subject = input
        .map(|i| join(quals(i).into_iter().map(String::from).chain(std::iter::once(i.get("Name").map(|n| st.line(n)).unwrap_or_default())).collect()))
        .unwrap_or_default();
    let forced: Vec<String> = match output.get("Qualifiers") {
        Some(Value::Array(q)) => q.iter().filter_map(|x| affixes.short(x)).collect(),
        _ => vec![],
    };
    let out_quals: Vec<String> = quals(output).into_iter().map(String::from).collect();
    // the stat lines the recipe adds, without counters like "Upgrades: 1 / 6"
    let stats: Vec<String> = match output.get("Lines") {
        Some(Value::Array(l)) => l
            .iter()
            .filter(|x| s(x, "key") != "Corrupted")
            .map(|x| st.line(x))
            .filter(|t| !t.trim().is_empty() && !(t.contains(" / ") && t.contains(':')))
            .take(2)
            .collect(),
        _ => vec![],
    };
    let detail = if stats.is_empty() { String::new() } else { format!(": {}", stats.join(", ")) };

    // portals and other fixed results
    if out_key.starts_with("strCubeNote") {
        return Some(st.get(out_key));
    }
    // the input item itself is changed
    if out_key.starts_with("strCubeOutput") {
        if !forced.is_empty() {
            return Some(join(vec![forced.join(" "), subject]));
        }
        let mut head = join(vec![action.unwrap_or("Change").to_string(), subject]);
        if out_key == "strCubeOutputUseTypeOfInput1" && !out_quals.is_empty() {
            let to = out_quals.join(" ");
            if to == "Crafted" && action == Some("Craft") {
                head = format!("Crafted {}", input.and_then(|i| i.get("Name")).map(|n| st.line(n)).unwrap_or_default());
            } else {
                head = format!("{} → {}", head, to);
            }
        }
        return Some(format!("{}{}", head, detail));
    }
    // a specific item comes out
    let out_code = output.get("Name").map(|n| s(n, "key")).unwrap_or("");
    let out_known = st.has(out_code);
    let out_name = join(out_quals.iter().cloned().chain(std::iter::once(output.get("Name").map(|n| st.line(n)).unwrap_or_default())).collect());
    if !forced.is_empty() {
        return Some(join(vec![forced.join(" "), out_name]));
    }
    let in_code = input.and_then(|i| i.get("Name")).map(|n| s(n, "key")).unwrap_or("");
    // arrows / bolts back out of the cube = refilled
    if matches!(out_code, "aqv" | "cqv") && in_code == out_code {
        return Some(format!("Refill {}", subject));
    }
    match action {
        Some("Craft") => Some(format!("{}{}", out_name, detail)),
        // nothing says what kind of recipe it is: it makes something
        None => Some(format!("Create {}{}", out_name, detail)),
        // same item back (reroll) or an internal name for the result: just say what is done to what
        Some(a) if out_name == subject || !out_known => Some(format!("{} {}", a, subject)),
        Some(a) => Some(format!("{} {} → {}", a, subject, out_name)),
    }
}
