//! Keeps generated assets original: prompts naming well-known characters,
//! franchises or brands are refused (the community guidelines don't allow
//! someone else's IP, and reviewers reject it), and "in the style of" a named
//! studio or artist gets a warning. This only catches the obvious cases; it's
//! a guard rail, not a clearance.
//!
//! A word that's yours or a false positive (a game about a lynx called
//! Sonic you own, say) can be let through with `--allow-name sonic`; the
//! sidecar records that you did.

/// Characters, franchises and brands. Matched as whole words, case-insensitively.
const REFUSE: &[&str] = &[
    // Games
    "mario",
    "luigi",
    "princess peach",
    "bowser",
    "yoshi",
    "donkey kong",
    "wario",
    "zelda",
    "hyrule",
    "triforce",
    "kirby",
    "pikachu",
    "pokemon",
    "pokémon",
    "pokeball",
    "poké ball",
    "eevee",
    "charizard",
    "sonic the hedgehog",
    "tails the fox",
    "knuckles the echidna",
    "pac-man",
    "pacman",
    "ms. pac-man",
    "tetris",
    "minecraft",
    "steve from minecraft",
    "fortnite",
    "roblox",
    "among us",
    "angry birds",
    "candy crush",
    "master chief",
    "halo spartan",
    "lara croft",
    "kratos",
    "solid snake",
    "mega man",
    "megaman",
    "street fighter",
    "ryu and ken",
    "mortal kombat",
    "final fantasy",
    "chocobo",
    "moogle",
    "cloud strife",
    "metroid",
    "samus",
    "animal crossing",
    "splatoon",
    "overwatch",
    "league of legends",
    "dota",
    "genshin",
    "hollow knight",
    "cuphead",
    "undertale",
    "five nights at freddy",
    "flappy bird",
    "frogger",
    "space invaders",
    "galaga",
    "dig dug",
    "q*bert",
    "crash bandicoot",
    "spyro",
    "rayman",
    "sackboy",
    "nintendo",
    "playstation",
    "xbox",
    "sega",
    "game boy",
    "gameboy",
    // Film, TV, comics
    "mickey mouse",
    "minnie mouse",
    "donald duck",
    "disney",
    "pixar",
    "winnie the pooh",
    "simba",
    "lion king",
    "buzz lightyear",
    "woody from toy story",
    "toy story",
    "shrek",
    "spongebob",
    "patrick star",
    "scooby-doo",
    "scooby doo",
    "bugs bunny",
    "looney tunes",
    "tom and jerry",
    "hello kitty",
    "sanrio",
    "totoro",
    "studio ghibli",
    "ghibli",
    "no-face",
    "naruto",
    "goku",
    "dragon ball",
    "one piece anime",
    "sailor moon",
    "doraemon",
    "astro boy",
    "gundam",
    "evangelion",
    "attack on titan",
    "demon slayer",
    "jujutsu kaisen",
    "my little pony",
    "transformers",
    "optimus prime",
    "power rangers",
    "teenage mutant ninja turtles",
    "ninja turtles",
    "peppa pig",
    "bluey",
    "paw patrol",
    "the simpsons",
    "bart simpson",
    "homer simpson",
    "rick and morty",
    "family guy",
    "smurf",
    "smurfs",
    "muppet",
    "kermit",
    "sesame street",
    "elmo",
    "garfield",
    "snoopy",
    "peanuts gang",
    "charlie brown",
    "star wars",
    "darth vader",
    "yoda",
    "baby yoda",
    "grogu",
    "stormtrooper",
    "lightsaber",
    "jedi",
    "sith",
    "r2-d2",
    "c-3po",
    "star trek",
    "harry potter",
    "hogwarts",
    "gandalf",
    "hobbit",
    "lord of the rings",
    "game of thrones",
    "marvel",
    "avengers",
    "spider-man",
    "spiderman",
    "iron man",
    "captain america",
    "thanos",
    "deadpool",
    "wolverine",
    "x-men",
    "dc comics",
    "batman",
    "superman",
    "wonder woman",
    "harley quinn",
    "aquaman",
    "james bond",
    "007",
    "indiana jones",
    "jurassic park",
    "godzilla",
    "king kong",
    "ghostbusters",
    "barbie",
    "ken doll",
    "lego",
    "hot wheels",
    "care bears",
    "teletubbies",
    "pingu",
    "wallace and gromit",
    "paddington",
    "peter rabbit",
    "moomin",
    "tintin",
    "asterix",
    "the grinch",
    "dr. seuss",
    "cat in the hat",
    "where's waldo",
    "pinocchio from disney",
    // Brands and logos
    "coca-cola",
    "coca cola",
    "pepsi",
    "mcdonald's",
    "mcdonalds",
    "ronald mcdonald",
    "burger king",
    "kfc",
    "starbucks",
    "nike",
    "adidas",
    "puma logo",
    "apple logo",
    "iphone",
    "google",
    "microsoft",
    "windows logo",
    "facebook",
    "instagram",
    "tiktok",
    "youtube",
    "twitter",
    "red bull",
    "monster energy",
    "ferrari",
    "lamborghini",
    "porsche",
    "bmw",
    "volkswagen",
    "harley-davidson",
    "rolex",
    "gucci",
    "louis vuitton",
    "chanel",
    "supreme logo",
    "michelin man",
    "pillsbury doughboy",
    "m&m's",
    "m&ms",
    "kool-aid man",
    "mr. peanut",
    "duolingo",
    "discord",
    "twitch",
    "netflix",
    "marlboro",
    "budweiser",
    "heineken",
    "nfl",
    "nba",
    "fifa",
    "olympic rings",
    "pringles",
    "oreo",
    "skittles",
    "doritos",
];

/// Style references to named studios and living artists: allowed, but a warning
/// (it pulls toward someone else's look, and reviewers may ask).
const WARN: &[&str] = &[
    "in the style of",
    "style of studio",
    "makoto shinkai",
    "hayao miyazaki",
    "miyazaki",
    "akira toriyama",
    "greg rutkowski",
    "artgerm",
    "loish",
    "beeple",
    "tim burton",
    "wes anderson",
    "moebius",
    "yoshitaka amano",
    "junji ito",
    "dreamworks",
    "cartoon network",
    "nickelodeon",
    "sanrio style",
    "pokemon style",
    "nintendo style",
];

#[derive(Debug, PartialEq)]
pub struct Verdict {
    pub refused: Vec<String>,
    pub warnings: Vec<String>,
    pub allowed: Vec<String>,
}

impl Verdict {
    pub fn ok(&self) -> bool {
        self.refused.is_empty()
    }
}

fn normalize(s: &str) -> String {
    s.to_lowercase().replace(['’', '‘'], "'")
}

/// Whether `needle` occurs in `hay` as whole words.
fn contains_words(hay: &str, needle: &str) -> bool {
    let is_word = |c: char| c.is_alphanumeric();
    let mut from = 0;
    while let Some(i) = hay[from..].find(needle) {
        let start = from + i;
        let end = start + needle.len();
        let before = hay[..start].chars().next_back();
        let after = hay[end..].chars().next();
        // Plurals and possessives of a name still count ("smurfs", "mario's").
        let after_ok = match after {
            None => true,
            Some(c) if !is_word(c) => true,
            Some('s') => hay[end + 1..].chars().next().is_none_or(|c| !is_word(c)),
            _ => false,
        };
        if before.is_none_or(|c| !is_word(c)) && after_ok {
            return true;
        }
        from = start + needle.len().max(1);
        if from >= hay.len() {
            break;
        }
    }
    false
}

/// Checks prompt text (the user's words and the style guide's) against the lists.
pub fn check(texts: &[&str], allow: &[String]) -> Verdict {
    let hay = normalize(&texts.join(" \n "));
    let allow: Vec<String> = allow.iter().map(|a| normalize(a)).collect();
    let mut v = Verdict { refused: Vec::new(), warnings: Vec::new(), allowed: Vec::new() };
    for name in REFUSE {
        if contains_words(&hay, name) {
            if allow.iter().any(|a| a == name || name.contains(a.as_str())) {
                v.allowed.push(name.to_string());
            } else {
                v.refused.push(name.to_string());
            }
        }
    }
    for name in WARN {
        if contains_words(&hay, name) && !allow.iter().any(|a| a == name) {
            v.warnings.push(name.to_string());
        }
    }
    v
}

/// `mb ip-check`: checks the text on stdin with no names allowed and prints
/// `{"ok", "refused", "warnings"}`. The server runs it on every prompt sent
/// to Maimbrain's keys, so both sides use this one list.
pub fn check_stdin() -> Result<(), String> {
    use std::io::Read;
    let mut text = String::new();
    std::io::stdin().take(1 << 20).read_to_string(&mut text).map_err(|e| format!("reading stdin: {e}"))?;
    let v = check(&[&text], &[]);
    println!("{}", serde_json::json!({ "ok": v.ok(), "refused": v.refused, "warnings": v.warnings }));
    Ok(())
}

pub fn refusal_message(v: &Verdict) -> String {
    format!(
        "the prompt names someone else's character, franchise or brand ({}). Maimbrain games must use original art and \
         sound (community guidelines; reviewers reject copies). Describe something original instead: what it looks \
         like, not what it's from. If a word is a false positive or you own the rights, pass --allow-name <word>.",
        v.refused.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_obvious_ip() {
        let v = check(&["a plumber like Mario jumping"], &[]);
        assert_eq!(v.refused, vec!["mario"]);
        assert!(!check(&["Pikachu-style yellow mouse"], &[]).ok());
        assert!(!check(&["two Smurfs"], &[]).ok(), "plurals count");
        assert!(!check(&["Darth Vader helmet"], &[]).ok());
        assert!(!check(&["COCA-COLA can"], &[]).ok());
    }

    #[test]
    fn allows_original_prompts_and_substrings() {
        for p in [
            "a moth with stained-glass wings",
            "marionette on strings",
            "a halogen lamp",
            "thorny vines",
            "an elsabeth",
            "a frozen lake",
            "the dark lord's minions",
            "one piece of cake",
            "the flash of a laser",
        ] {
            assert!(check(&[p], &[]).ok(), "{p}");
        }
        assert!(check(&["tetromino blocks falling"], &[]).ok());
    }

    #[test]
    fn allow_name_overrides_and_is_recorded() {
        let v = check(&["sonic the hedgehog fan art"], &["sonic the hedgehog".into()]);
        assert!(v.ok());
        assert_eq!(v.allowed, vec!["sonic the hedgehog"]);
    }

    #[test]
    fn warns_on_named_styles() {
        let v = check(&["a forest in the style of Hayao Miyazaki"], &[]);
        assert!(v.ok());
        assert!(v.warnings.contains(&"hayao miyazaki".to_string()));
    }
}
