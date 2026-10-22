//! Port of the frozen delivery plate_semantic_v1_5 rules (Apache-2.0).
//! Only original text enters parsing. Length never resolves an ambiguous alias.
use super::unicode::UnicodePolicy;
use crate::error::{Result, WeTextError};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

pub(super) const PROVINCES: &str = "京津沪渝冀豫云辽黑湘皖鲁新苏浙赣鄂桂甘晋蒙陕吉闽贵粤青藏川宁琼";
pub(super) const NEUTRAL: &[&str] = &[
    "不是旁边那辆车",
    "不是旁边那辆",
    "不是让你改车牌",
    "不要记错了",
    "不要记错",
    "别记错了",
    "别记错",
];
const FILLERS: &[&str] = &[
    "我分开说",
    "再是",
    "中间是",
    "剩下的是",
    "你听着啊",
    "等一下我想想",
    "稍等",
    "我看看",
    "那个",
    "我看一下",
    "第一个字符是",
    "后面的字符是",
    "接下来是",
    "接着是",
    "前面是",
    "后面是",
    "最后是",
    "我报慢一点",
    "我慢点报",
    "然后是",
    "再一个",
    "还有一个",
    "呃",
    "嗯",
];

macro_rules! rules {
    ($($name:ident: $pattern:literal),* $(,)?) => {
        pub(super) struct Rules { $(pub $name: Regex),* }
        impl Rules { fn new() -> Result<Self> { Ok(Self { $($name: compile($pattern)?),* }) } }
    };
}
rules! {
    stop: r"^(?:记下来了吗|记下来了|记下来吧|记录了吗|记录好了|记好了|记住了吗|记一下|开的是|开过去)",
    anchor: r"^([A-Z]{1,2})的([A-Z])",
    identity: r"^([A-Z])[|]*(?:也[|]*就[|]*是|就[|]*是)[|]*字[|]*母[|]*([A-Z])",
    alias_identity: r"^(?:也就是|也就是字母|就是字母|也就是数字|就是数字)(?:字母|数字)?([A-Z零〇一二两三四五六七八九幺0-9])",
    kind: r"^(大[|]*写[|]*字[|]*母|字[|]*母|大[|]*写[|]*的|大[|]*写|数[|]*字)",
    number: r"^[零〇一二两三四五六七八九十百千万]+",
    quantity: r"^(?:([一二两三四五六七八九1-9])[|]*个|([仨俩双])|([一二两三四五六七八九1-9])[|]*(?:连|次|遍))",
    quantity_after: r"^(?:重复[|]*)?([一二两三四五六七八九1-9])[|]*(?:次|遍)",
    valid: r"^[京津沪渝冀豫云辽黑湘皖鲁新苏浙赣鄂桂甘晋蒙陕吉闽贵粤青藏川宁琼][A-HJ-NP-Z][A-HJ-NP-Z0-9学]+$",
    energy: r"^(?:[DF][A-HJ-NP-Z0-9][0-9]{4}|[0-9]{5}[DF])$",
    context: r"^(?:[零〇一二两三四五六七八九十百0-9]+(?:个)?(?:小时|分钟|秒钟|点|天|号车位)|(?:数字|字母).*(?:我念作|读作))",
    hesitation: r"^什么.{1,24}?来着[|]*哦对[|]*",
    correction: r"不对|不是|说错|改成|改为|更正|纠正|从头报|重头报|重新报|再完整报|我重报|还是|或者|不确定|可能|作废|取消|记不清",
    correction_at: r"^(?:不对|不是|说错|改成|改为|更正|纠正|从头报|重头报|重新报|再完整报|我重报|还是|或者|不确定|可能|作废|取消|记不清|第[一二三四五六七八1-8]位|最后(?:一个|一位|一位字符|一个字符))",
    replacement: r"(?:第([一二三四五六七八1-8])位|最后(?:一[|]*个字符|一位字符|一[|]*个|一位)?)(?:不是(?P<old>[^|]{1,12})[|]*[而]?是|(?:应该)?是|改成|改为)",
    negative: r"不是(?:字母|数字)?[A-Z零一二三四五六七八九0-9]",
    uncertain: r"(?:不是|可能是|不确定是|也许是|不要登记)[|]*$",
    restart: r"不对|不是|改成|改为|从头报|重头报|重新报|再完整报|我重报",
    nonplate: r"(?:发动机号|保单号|手机号码|手机号|电话号码|不是车牌)(?:是|为|号码是|[|])*$",
    following_negative: r"^[|]*不是(?:字母|数字)?[A-Z零一二三四五六七八九0-9]",
    markers: r"第[一二三四五六七八1-8]位|最后",
    correction_filler: r"不对|说错了|我|是|[|]",
    explicit_plate: r"(?:车牌|牌照)(?:的)?[|]*$",
    competing: r"[0-9零〇一二两三四五六七八九十百千万]|金额|单号|电话|保单|发动机",
    restart_bridge: r"再完整报一遍|再完整报|没说完|从头报|重头报|重新报|我重报|不对|不是|改成|改为|我|是|[|]",
}

fn compile(pattern: &str) -> Result<Regex> {
    Regex::new(pattern).map_err(|e| WeTextError::InvalidProfile(e.to_string()))
}
fn punctuated(text: &str) -> String {
    text.chars()
        .map(|c| regex::escape(&c.to_string()))
        .collect::<Vec<_>>()
        .join("[|]*")
}
pub(super) fn skip(s: &str, mut p: usize) -> usize {
    while s.as_bytes().get(p) == Some(&b'|') {
        p += 1;
    }
    p
}
pub(super) fn prefix(s: &str, end: usize, count: usize) -> &str {
    let start = s[..end]
        .char_indices()
        .rev()
        .nth(count - 1)
        .map_or(0, |(i, _)| i);
    &s[start..end]
}
fn nchar(s: &str) -> usize {
    s.chars().count()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Lexicon {
    province_names: BTreeMap<String, String>,
    digit_aliases: BTreeMap<String, Vec<String>>,
    letter_aliases: BTreeMap<String, Vec<String>>,
    excluded_positive_letters: Vec<String>,
    note: String,
}
struct Alias {
    key: String,
    pattern: Regex,
    values: BTreeSet<char>,
}
pub(super) struct Semantics {
    pub lexicon_sha256: String,
    pub unicode: UnicodePolicy,
    pub rules: Rules,
    digits: BTreeMap<char, char>,
    aliases: Vec<Alias>,
    heads: Vec<(String, char)>,
    anchors: Vec<(Regex, char)>,
    fillers: Vec<Regex>,
}
#[derive(Clone)]
pub(super) struct Candidate {
    pub value: String,
    pub start: usize,
    pub end: usize,
    pub ambiguous: bool,
    pub unresolved: bool,
    pub complete: bool,
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Status {
    Complete,
    Multiple,
    Other,
}
pub(super) struct Parsed {
    pub text: String,
    pub candidates: Vec<Candidate>,
    pub status: Status,
    pub plate: Option<String>,
}
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Any,
    Digit,
    Letter,
}
struct Atom {
    value: String,
    end: usize,
    ambiguous: bool,
}

impl Semantics {
    pub fn check_input(&self, raw: &str) -> Result<()> {
        let normalized = self.unicode.view(raw);
        let mut pos = 0;
        while pos < normalized.len() {
            let mut end = pos;
            let mut nesting = 0;
            while let Some(kind) = self.rules.kind.find(&normalized[end..]) {
                nesting += 1;
                if nesting > 64 {
                    return Err(WeTextError::InvalidInput(
                        "explicit symbol-kind nesting exceeds 64".into(),
                    ));
                }
                end = skip(&normalized, end + kind.end());
            }
            pos = if end > pos {
                end
            } else {
                pos + normalized[pos..].chars().next().map_or(1, char::len_utf8)
            };
        }
        Ok(())
    }
    pub fn new(json: &str) -> Result<Self> {
        let lex: Lexicon =
            serde_json::from_str(json).map_err(|e| WeTextError::InvalidProfile(e.to_string()))?;
        if lex.excluded_positive_letters != ["I", "O"]
            || lex.note.is_empty()
            || lex.province_names.len() != 31
        {
            return Err(WeTextError::InvalidProfile(
                "unsupported plate lexicon policy".into(),
            ));
        }
        let unicode = UnicodePolicy::load()?;
        let mut digits: BTreeMap<_, _> = "零〇一二两三四五六七八九幺"
            .chars()
            .zip("0012234567891".chars())
            .collect();
        let mut values: BTreeMap<String, BTreeSet<char>> = BTreeMap::new();
        for collection in [&lex.digit_aliases, &lex.letter_aliases] {
            for (value, words) in collection {
                let symbol = value
                    .chars()
                    .next()
                    .filter(|_| nchar(value) == 1)
                    .ok_or_else(|| {
                        WeTextError::InvalidProfile("alias must name one symbol".into())
                    })?;
                for word in words {
                    let key: String = unicode
                        .upper(word)
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect();
                    values.entry(key).or_default().insert(symbol);
                }
            }
        }
        for (value, words) in &lex.digit_aliases {
            let digit = value
                .chars()
                .next()
                .ok_or_else(|| WeTextError::InvalidProfile("empty digit".into()))?;
            for word in words {
                if word != "勾" && word != "钩" && nchar(word) == 1 {
                    if let Some(c) = word.chars().next() {
                        digits.insert(c, digit);
                    }
                }
            }
        }
        for (word, symbols) in [
            ("勾", "9J"),
            ("钩", "9J"),
            ("衣", "E1"),
            ("啊", "R2"),
            ("欧", "O0"),
        ] {
            values.insert(word.into(), symbols.chars().collect());
        }
        let mut aliases = Vec::new();
        for (key, values) in values {
            let pattern = if key == "DOUBLEU" {
                "DOUBLE[|]*U".into()
            } else if key.is_ascii() {
                regex::escape(&key)
            } else {
                punctuated(&key)
            };
            aliases.push(Alias {
                key,
                pattern: compile(&format!("^(?:{pattern})"))?,
                values,
            });
        }
        aliases.sort_by_key(|alias| std::cmp::Reverse(nchar(&alias.key)));
        let mut heads: Vec<_> = PROVINCES.chars().map(|c| (c.to_string(), c)).collect();
        for (short, name) in &lex.province_names {
            let province = short
                .chars()
                .next()
                .filter(|&c| nchar(short) == 1 && PROVINCES.contains(c))
                .ok_or_else(|| WeTextError::InvalidProfile("invalid province".into()))?;
            heads.push((name.clone(), province));
        }
        heads.sort_by_key(|(name, _)| std::cmp::Reverse(nchar(name)));
        let mut anchor_text = vec![
            ("两个肚子的B", 'B'),
            ("一个肚子的D", 'D'),
            ("不带钩的G", 'G'),
            ("带钩的J", 'J'),
            ("叉那个X", 'X'),
            ("哥的G", 'G'),
            ("二B铅笔的B", 'B'),
            ("大写的X", 'X'),
            ("两竖一横的H", 'H'),
        ];
        anchor_text.sort_by_key(|(text, _)| std::cmp::Reverse(nchar(text)));
        let anchors = anchor_text
            .into_iter()
            .map(|(text, value)| Ok((compile(&format!("^{}", punctuated(text)))?, value)))
            .collect::<Result<_>>()?;
        let fillers = FILLERS
            .iter()
            .map(|text| compile(&format!("^{}", punctuated(text))))
            .collect::<Result<_>>()?;
        use sha2::Digest;
        let lexicon_sha256 = format!("{:x}", sha2::Sha256::digest(json.as_bytes()));
        Ok(Self {
            lexicon_sha256,
            unicode,
            rules: Rules::new()?,
            digits,
            aliases,
            heads,
            anchors,
            fillers,
        })
    }

    fn digit(&self, c: char) -> char {
        self.digits.get(&c).copied().unwrap_or(c)
    }
    fn cardinal(&self, text: &str) -> Option<(String, usize)> {
        if text == "十" {
            return Some(("10".into(), "十".len()));
        }
        let chars: Vec<_> = text.chars().collect();
        for count in (2..=chars.len().min(15)).rev() {
            let part = &chars[..count];
            if !part.iter().any(|c| "十百千万".contains(*c)) {
                continue;
            }
            let (mut total, mut section, mut digit) = (0u32, 0u32, 0u32);
            for &c in part {
                if let Some(d) = self.digits.get(&c).and_then(|c| c.to_digit(10)) {
                    digit = d;
                } else if c == '万' {
                    total += (section + digit) * 10000;
                    section = 0;
                    digit = 0;
                } else if let Some(unit) = match c {
                    '十' => Some(10),
                    '百' => Some(100),
                    '千' => Some(1000),
                    _ => None,
                } {
                    section += digit.max(1) * unit;
                    digit = 0;
                } else {
                    return None;
                }
            }
            let value = total + section + digit;
            if value == 0 || value >= 100000 {
                continue;
            }
            let canonical = cn_read(value);
            let normalized: String = part
                .iter()
                .map(|&c| {
                    if c == '两' {
                        '二'
                    } else if c == '〇' {
                        '零'
                    } else {
                        c
                    }
                })
                .collect();
            if normalized == canonical
                || canonical.starts_with('十') && normalized == format!("一{canonical}")
            {
                return Some((value.to_string(), part.iter().map(|c| c.len_utf8()).sum()));
            }
        }
        None
    }

    fn atom(&self, s: &str, pos: usize, kind: Kind, allow_cardinal: bool) -> Option<Atom> {
        let p = skip(s, pos);
        let tail = s.get(p..)?;
        let c = tail.chars().next()?;
        if self.rules.stop.is_match(tail) {
            return None;
        }
        for (pattern, value) in &self.anchors {
            if let Some(m) = pattern.find(tail) {
                return Some(Atom {
                    value: value.to_string(),
                    end: p + m.end(),
                    ambiguous: false,
                });
            }
        }
        if let Some(m) = self.rules.anchor.captures(tail) {
            if m[1].contains(&m[2]) {
                return Some(Atom {
                    value: m[2].into(),
                    end: p + m.get(0)?.end(),
                    ambiguous: false,
                });
            }
        }
        if let Some(m) = self.rules.identity.captures(tail) {
            if kind == Kind::Digit {
                return None;
            }
            return Some(Atom {
                value: if m[1] == m[2] {
                    m[2].into()
                } else {
                    "?".into()
                },
                end: p + m.get(0)?.end(),
                ambiguous: m[1] != m[2],
            });
        }
        for alias in &self.aliases {
            if let Some(m) = alias.pattern.find(tail) {
                let q = skip(s, p + m.end());
                if let Some(explain) = self.rules.alias_identity.captures(&s[q..]) {
                    let c = explain[1].chars().next()?;
                    return Some(Atom {
                        value: self.digit(c).to_string(),
                        end: q + explain.get(0)?.end(),
                        ambiguous: false,
                    });
                }
            }
        }
        if let Some(m) = self.rules.kind.captures(tail) {
            let k = if m[1].replace('|', "") == "数字" {
                Kind::Digit
            } else {
                Kind::Letter
            };
            let result = self.atom(s, p + m.get(0)?.end(), k, false)?;
            if !result.ambiguous
                && (k == Kind::Digit && !result.value.chars().all(|c| c.is_ascii_digit())
                    || k == Kind::Letter
                        && !(nchar(&result.value) == 1
                            && result.value.chars().all(|c| c.is_ascii_uppercase())))
            {
                return None;
            }
            return Some(result);
        }
        if kind != Kind::Letter && allow_cardinal {
            if let Some(m) = self.rules.number.find(tail) {
                if let Some((value, used)) = self.cardinal(m.as_str()) {
                    let rest = &m.as_str()[used..];
                    let last = m.as_str()[..used].chars().next_back()?;
                    let ambiguous = "百千万".contains(last)
                        && rest
                            .chars()
                            .next()
                            .is_some_and(|c| "一二两三四五六七八九".contains(c));
                    return Some(Atom {
                        value: if ambiguous { "?".into() } else { value },
                        end: p + if ambiguous { m.end() } else { used },
                        ambiguous,
                    });
                }
            }
        }
        for alias in &self.aliases {
            if alias.key == "DOUBLEU" || alias.key == "WHY" {
                if let Some(m) = alias.pattern.find(tail) {
                    if kind == Kind::Digit {
                        return None;
                    }
                    return Some(Atom {
                        value: alias.values.iter().next()?.to_string(),
                        end: p + m.end(),
                        ambiguous: false,
                    });
                }
            }
        }
        if c.is_ascii_uppercase() {
            return (kind != Kind::Digit).then(|| Atom {
                value: c.to_string(),
                end: p + 1,
                ambiguous: false,
            });
        }
        if self.digits.contains_key(&c) || c.is_ascii_digit() {
            return (kind != Kind::Letter).then(|| Atom {
                value: self.digit(c).to_string(),
                end: p + c.len_utf8(),
                ambiguous: false,
            });
        }
        for alias in &self.aliases {
            if let Some(m) = alias.pattern.find(tail) {
                let values: Vec<_> = alias
                    .values
                    .iter()
                    .copied()
                    .filter(|c| match kind {
                        Kind::Any => true,
                        Kind::Digit => c.is_ascii_digit(),
                        Kind::Letter => c.is_ascii_alphabetic(),
                    })
                    .collect();
                if values.is_empty() {
                    return None;
                }
                return Some(Atom {
                    value: if values.len() == 1 {
                        values[0].to_string()
                    } else {
                        "?".into()
                    },
                    end: p + m.end(),
                    ambiguous: values.len() > 1,
                });
            }
        }
        None
    }

    fn unit(&self, s: &str, pos: usize) -> Option<Atom> {
        let p = skip(s, pos);
        if let Some(m) = self.rules.quantity.captures(&s[p..]) {
            let count = if let Some(c) = m.get(1).or_else(|| m.get(3)) {
                self.digit(c.as_str().chars().next()?).to_digit(10)?
            } else {
                match m.get(2)?.as_str() {
                    "仨" => 3,
                    "俩" | "双" => 2,
                    _ => return None,
                }
            };
            if let Some(mut atom) = self.atom(s, p + m.get(0)?.end(), Kind::Any, false) {
                if nchar(&atom.value) == 1 {
                    atom.value = atom.value.repeat(count as usize);
                    return Some(atom);
                }
            }
        }
        let mut atom = self.atom(s, p, Kind::Any, true)?;
        let q = skip(s, atom.end);
        if let Some(m) = self.rules.quantity_after.captures(&s[q..]) {
            if nchar(&atom.value) == 1 {
                let count = self.digit(m[1].chars().next()?).to_digit(10)?;
                atom.value = atom.value.repeat(count as usize);
                atom.end = q + m.get(0)?.end();
            }
        }
        Some(atom)
    }

    fn valid(&self, value: &str) -> bool {
        if !self.rules.valid.is_match(value) {
            return false;
        }
        let body: String = value.chars().skip(2).collect();
        if nchar(&body) == 5 {
            return if let Some(digits) = body.strip_suffix('学') {
                digits.chars().all(|c| c.is_ascii_digit())
            } else {
                !body.contains('学')
                    && body.chars().filter(|c| c.is_ascii_alphabetic()).count() <= 2
            };
        }
        self.rules.energy.is_match(&body)
    }

    fn head(&self, s: &str, p: usize) -> Option<Atom> {
        for (name, province) in &self.heads {
            if s[p..].starts_with(name) {
                let mut q = skip(s, p + name.len());
                for extra in [
                    "省".into(),
                    format!("那个{province}"),
                    format!("的{province}"),
                ] {
                    if s[q..].starts_with(&extra) {
                        q = skip(s, q + extra.len());
                    }
                }
                if let Some(atom) = self.atom(s, q, Kind::Letter, false) {
                    if nchar(&atom.value) == 1 {
                        let raw = &s[q..atom.end];
                        if self.aliases.iter().any(|a| a.key == raw)
                            && atom.end < s.len()
                            && s.as_bytes()[atom.end] != b'|'
                            && self.atom(s, atom.end, Kind::Any, true).is_none()
                            && !FILLERS.iter().any(|f| s[atom.end..].starts_with(f))
                        {
                            continue;
                        }
                        return Some(Atom {
                            value: format!("{province}{}", atom.value),
                            ..atom
                        });
                    }
                }
            }
        }
        None
    }

    pub fn scan(&self, s: &str, start: usize) -> Option<Candidate> {
        let h = self.head(s, start)?;
        let mut value = h.value;
        let mut p = h.end;
        let mut end = p;
        let mut ambiguous = h.ambiguous;
        for _ in 0..80 {
            let q = skip(s, p);
            if q >= s.len() {
                break;
            }
            if s[q..].starts_with('学') && nchar(&value) == 6 {
                value.push('学');
                p = q + '学'.len_utf8();
                end = p;
                continue;
            }
            if self.rules.stop.is_match(&s[q..])
                || nchar(&value) >= 7 && self.rules.context.is_match(&s[q..])
            {
                break;
            }
            if nchar(&value) == 2 {
                if let Some(m) = self.rules.hesitation.find(&s[q..]) {
                    p = q + m.end();
                    continue;
                }
            }
            if self.head(s, q).is_some()
                || self.rules.correction_at.is_match(&s[q..])
                || nchar(&value) >= 7 && s[q..].starts_with("最后是")
            {
                break;
            }
            if let Some(atom) = self.unit(s, q) {
                value.push_str(&atom.value);
                ambiguous |= atom.ambiguous;
                p = atom.end;
                end = p;
                continue;
            }
            if nchar(&value) < 7 {
                if let Some(m) = self.fillers.iter().find_map(|f| f.find(&s[q..])) {
                    p = q + m.end();
                    continue;
                }
            }
            break;
        }
        Some(Candidate {
            value,
            start,
            end,
            ambiguous,
            unresolved: false,
            complete: false,
        })
    }

    pub fn correct(&self, candidate: &mut Candidate, tail: &str) -> (bool, usize) {
        let mut chars: Vec<_> = candidate.value.chars().collect();
        let mut changed = false;
        let mut used = 0;
        for m in self.rules.replacement.captures_iter(tail) {
            let Some(found) = m.get(0) else { continue };
            if ["电话号码", "手机号", "发动机", "保单"]
                .iter()
                .any(|word| tail[..found.start()].contains(word))
            {
                continue;
            }
            let Some(atom) = self.atom(tail, found.end(), Kind::Any, false) else {
                candidate.ambiguous = true;
                continue;
            };
            if nchar(&atom.value) != 1 || atom.ambiguous {
                candidate.ambiguous = true;
                continue;
            }
            let index = m
                .get(1)
                .and_then(|n| n.as_str().chars().next())
                .and_then(|c| self.digit(c).to_digit(10))
                .map(|n| n as usize - 1)
                .or_else(|| chars.len().checked_sub(1));
            let Some(index) = index.filter(|&i| i < chars.len()) else {
                candidate.ambiguous = true;
                continue;
            };
            if self.atom(tail, atom.end, Kind::Any, false).is_some() {
                candidate.ambiguous = true;
                continue;
            }
            if let Some(old) = m.name("old") {
                let previous = self.atom(old.as_str(), 0, Kind::Any, false);
                if !previous.is_some_and(|a| {
                    a.value == chars[index].to_string() && a.end == old.as_str().len()
                }) {
                    candidate.ambiguous = true;
                    continue;
                }
            }
            if let Some(value) = atom.value.chars().next() {
                chars[index] = value;
                changed = true;
                used = used.max(atom.end);
            }
        }
        candidate.value = chars.into_iter().collect();
        (changed, used)
    }

    pub fn parse(&self, raw: &str) -> Parsed {
        let text = self.unicode.view(raw);
        let s = &text;
        let mut candidates = Vec::new();
        let mut p = 0;
        while p < s.len() {
            if let Some(candidate) = self.scan(s, p) {
                p = candidate.end.max(p + 1);
                candidates.push(candidate);
            } else {
                p += s[p..].chars().next().map_or(1, char::len_utf8);
            }
        }
        for i in 0..candidates.len() {
            let stop = candidates.get(i + 1).map_or(s.len(), |c| c.start);
            let c = &mut candidates[i];
            let tail = &s[c.end..stop];
            let (changed, used) = self.correct(c, tail);
            let mut clean = tail.to_string();
            for word in NEUTRAL {
                clean = clean.replace(word, "");
            }
            if changed {
                let used_chars = tail[..used].chars().count();
                let start = clean
                    .char_indices()
                    .nth(used_chars)
                    .map_or(clean.len(), |(i, _)| i);
                clean = self
                    .rules
                    .negative
                    .replace_all(&clean[start..], "")
                    .into_owned();
            }
            c.unresolved = self.rules.correction.is_match(&clean)
                || self.rules.uncertain.is_match(prefix(s, c.start, 16));
            c.complete = self.valid(&c.value) && !c.ambiguous;
        }
        let mut active: Vec<&Candidate> = Vec::new();
        for candidate in &candidates {
            if let Some(previous) = active.last() {
                let mut bridge = s[previous.end..candidate.start].to_string();
                for word in NEUTRAL {
                    bridge = bridge.replace(word, "");
                }
                if self.rules.restart.is_match(&bridge) || previous.value == candidate.value {
                    active.clear();
                }
            }
            active.push(candidate);
        }
        let nonplate = candidates
            .first()
            .is_some_and(|c| self.rules.nonplate.is_match(prefix(s, c.start, 30)));
        let (status, plate) = if nonplate
            || active.is_empty()
            || active
                .iter()
                .any(|c| c.ambiguous || c.unresolved || !c.complete)
        {
            (Status::Other, None)
        } else {
            let mut values = Vec::new();
            for c in active {
                if !values.contains(&c.value) {
                    values.push(c.value.clone());
                }
            }
            if values.len() == 1 {
                (Status::Complete, values.pop())
            } else {
                (Status::Multiple, None)
            }
        };
        Parsed {
            text,
            candidates,
            status,
            plate,
        }
    }
}

fn cn_read(mut n: u32) -> String {
    if n == 0 {
        return "零".into();
    }
    let digits: Vec<_> = "零一二三四五六七八九".chars().collect();
    let mut output = String::new();
    let mut pending = false;
    for (unit, label) in [
        (10000, "万"),
        (1000, "千"),
        (100, "百"),
        (10, "十"),
        (1, ""),
    ] {
        let d = n / unit;
        n %= unit;
        if d > 0 {
            if pending {
                output.push('零');
            }
            output.push(digits[d as usize]);
            output.push_str(label);
            pending = false;
        } else if !output.is_empty() && n > 0 {
            pending = true;
        }
    }
    output
        .strip_prefix("一十")
        .map_or_else(|| output.clone(), |rest| format!("十{rest}"))
}
