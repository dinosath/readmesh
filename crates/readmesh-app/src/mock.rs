//! Deterministic mock catalog backing the app without any backend.
//!
//! Implements [`ContentRepository`] with a fixed set of novels, chapters and
//! generated chapter text. Everything is derived from stable seeds so the
//! data is identical across runs and platforms, which keeps UI behavior
//! reproducible and tests reliable.

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};
use readmesh_core::{Chapter, ChapterId, Novel, NovelId, NovelStatus, PluginId};

use crate::repository::{ContentRepository, SourceInfo};

/// Extra presentation metadata not (yet) part of the `readmesh-core` domain model.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NovelMeta {
    pub alt_titles: Vec<String>,
    pub artist: Option<String>,
}

struct NovelSpec {
    title: &'static str,
    alt_titles: &'static [&'static str],
    author: &'static str,
    artist: Option<&'static str>,
    genres: &'static [&'static str],
    status: NovelStatus,
    summary: &'static str,
    source: usize,
    chapters: u32,
    added_days_ago: i64,
    updated_days_ago: i64,
}

const SOURCES: [(&str, &str, &str); 3] = [
    ("meshpress", "MeshPress", "1.4.2"),
    ("lantern", "Lantern Books", "0.9.1"),
    ("archive-peer", "Archive Peer", "2.0.0"),
];

const NOVELS: &[NovelSpec] = &[
    NovelSpec {
        title: "Moonlit Blade of the Fallen Star",
        alt_titles: &["Hoshi no Gekkou", "Blade of the Fallen Star"],
        author: "Kazehara Ren",
        artist: Some("Mio Takanashi"),
        genres: &["Fantasy", "Action", "Adventure"],
        status: NovelStatus::Ongoing,
        summary: "When a star falls into the Sea of Embers, a disgraced swordswoman \
                  inherits its light — and the hunt begins. Every sect in the Nine \
                  Provinces wants the blade, and none of them know what it cost her.",
        source: 0,
        chapters: 38,
        added_days_ago: 340,
        updated_days_ago: 1,
    },
    NovelSpec {
        title: "The Alchemist's Last Experiment",
        alt_titles: &[],
        author: "V. A. Holloway",
        artist: None,
        genres: &["Fantasy", "Mystery"],
        status: NovelStatus::Completed,
        summary: "A royal alchemist is found dead beside a transmutation circle that \
                  should not work. Her apprentice has seven days to finish the \
                  experiment — or be finished by it.",
        source: 1,
        chapters: 24,
        added_days_ago: 500,
        updated_days_ago: 120,
    },
    NovelSpec {
        title: "Reborn as a Dungeon Botanist",
        alt_titles: &["Dungeon no Shokubutsugaku"],
        author: "Pip Marlowe",
        artist: Some("Aoi Shin"),
        genres: &["Fantasy", "Comedy", "Adventure"],
        status: NovelStatus::Ongoing,
        summary: "Reincarnated with nothing but a botany textbook and opinions, \
                  Ren decides the fastest way to conquer a dungeon is to landscape it.",
        source: 0,
        chapters: 41,
        added_days_ago: 210,
        updated_days_ago: 2,
    },
    NovelSpec {
        title: "Starward Drift",
        alt_titles: &[],
        author: "Ikenna Osei",
        artist: None,
        genres: &["Sci-Fi", "Adventure"],
        status: NovelStatus::Ongoing,
        summary: "A salvage crew picks up a distress call from a ship that was lost \
                  two hundred years ago. The cargo manifest lists one item: a sunrise.",
        source: 2,
        chapters: 19,
        added_days_ago: 150,
        updated_days_ago: 4,
    },
    NovelSpec {
        title: "Signal in the Static",
        alt_titles: &["The Static Speaks"],
        author: "Mara Voss",
        artist: None,
        genres: &["Sci-Fi", "Mystery"],
        status: NovelStatus::Hiatus,
        summary: "Every radio in the city played the same six seconds of static last \
                  night. A late-night DJ realizes it was a message — and that it was \
                  meant for her.",
        source: 2,
        chapters: 12,
        added_days_ago: 400,
        updated_days_ago: 200,
    },
    NovelSpec {
        title: "The Clockwork Duchess",
        alt_titles: &[],
        author: "Eleanora Finch",
        artist: Some("J. Beaumont"),
        genres: &["Romance", "Fantasy"],
        status: NovelStatus::Completed,
        summary: "Betrothed to a duke who is rarely seen without gloves, a watchmaker's \
                  daughter discovers the household runs on more than etiquette.",
        source: 1,
        chapters: 30,
        added_days_ago: 620,
        updated_days_ago: 90,
    },
    NovelSpec {
        title: "Petals After the Rain",
        alt_titles: &["Ameagari no Hanabira"],
        author: "Yui Hanasaki",
        artist: None,
        genres: &["Romance", "Slice of Life", "Drama"],
        status: NovelStatus::Ongoing,
        summary: "A florist who can hear what flowers remember meets a customer who \
                  keeps buying bouquets for someone who never comes.",
        source: 0,
        chapters: 16,
        added_days_ago: 80,
        updated_days_ago: 6,
    },
    NovelSpec {
        title: "A Cat's Guide to the Apocalypse",
        alt_titles: &[],
        author: "T. Whiskers",
        artist: Some("Mio Takanashi"),
        genres: &["Comedy", "Adventure"],
        status: NovelStatus::Ongoing,
        summary: "The world ended on a Tuesday. The cat, unimpressed, has notes. \
                  A survival guide narrated by humanity's most reluctant witness.",
        source: 2,
        chapters: 27,
        added_days_ago: 190,
        updated_days_ago: 3,
    },
    NovelSpec {
        title: "The Silent Archive",
        alt_titles: &[],
        author: "Mara Voss",
        artist: None,
        genres: &["Mystery", "Horror"],
        status: NovelStatus::Completed,
        summary: "An archivist cataloguing a shuttered library finds a shelf of books \
                  that were never written — including one with her name on the spine.",
        source: 1,
        chapters: 22,
        added_days_ago: 700,
        updated_days_ago: 300,
    },
    NovelSpec {
        title: "Ashes of the Ninth Gate",
        alt_titles: &["Ninth Gate"],
        author: "Kazehara Ren",
        artist: None,
        genres: &["Fantasy", "Action", "Drama"],
        status: NovelStatus::Ongoing,
        summary: "Eight gates sealed the old war. The ninth was never closed. \
                  A gatekeeper's apprentice inherits the watch on the night it opens.",
        source: 0,
        chapters: 33,
        added_days_ago: 280,
        updated_days_ago: 5,
    },
    NovelSpec {
        title: "Second Breakfast at the End of the World",
        alt_titles: &[],
        author: "Pip Marlowe",
        artist: None,
        genres: &["Comedy", "Slice of Life", "Fantasy"],
        status: NovelStatus::Ongoing,
        summary: "The prophecy says the world ends in thirty days. The innkeeper's \
                  only question is whether the heroes will settle their tab first.",
        source: 1,
        chapters: 14,
        added_days_ago: 60,
        updated_days_ago: 1,
    },
    NovelSpec {
        title: "The Cartographer's Daughter",
        alt_titles: &["Mapmaker's Heir"],
        author: "Ikenna Osei",
        artist: Some("Aoi Shin"),
        genres: &["Adventure", "Romance"],
        status: NovelStatus::Dropped,
        summary: "Her father mapped every coast except one. Armed with his unfinished \
                  atlas, she sets out to draw the shoreline he was forbidden to see.",
        source: 2,
        chapters: 18,
        added_days_ago: 450,
        updated_days_ago: 260,
    },
    NovelSpec {
        title: "Ghost Frequency",
        alt_titles: &[],
        author: "V. A. Holloway",
        artist: None,
        genres: &["Horror", "Mystery"],
        status: NovelStatus::Ongoing,
        summary: "A paranormal debunker buys a shortwave radio at an estate sale. \
                  It only picks up one station, and the station keeps describing \
                  her living room.",
        source: 1,
        chapters: 9,
        added_days_ago: 40,
        updated_days_ago: 2,
    },
    NovelSpec {
        title: "Tides of the Hollow Sea",
        alt_titles: &["Hollow Sea"],
        author: "Yui Hanasaki",
        artist: Some("J. Beaumont"),
        genres: &["Fantasy", "Adventure", "Drama"],
        status: NovelStatus::Ongoing,
        summary: "The tide went out one night and never came back. A tide-callers' \
                  guild sends its last apprentice to walk the dry seabed and ask why.",
        source: 0,
        chapters: 25,
        added_days_ago: 130,
        updated_days_ago: 7,
    },
];

/// Word pool for deterministic chapter text generation.
const WORDS: &[&str] = &[
    "wind", "lantern", "road", "whisper", "stone", "river", "shadow", "ember", "gate", "ink",
    "tide", "glass", "root", "star", "ash", "bell", "map", "thorn", "mist", "crown", "salt",
    "feather", "hollow", "light", "letter", "seed", "wall", "song", "thread", "door", "field",
    "tower", "promise", "silence", "branch", "harbor",
];

/// Tiny deterministic PRNG (xorshift64*) seeded from a hash.
struct Prng(u64);

impl Prng {
    fn from_seed(seed: &[u8]) -> Self {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&seed[..8]);
        let state = u64::from_le_bytes(buf) | 1; // avoid the all-zero state
        Self(state)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn title_case(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Deterministic chapter text: heading + paragraphs of generated prose.
fn generate_content(seed: &[u8], chapter_index: u32) -> String {
    let mut rng = Prng::from_seed(seed);
    let mut out = String::with_capacity(16 * 1024);
    let paragraphs = 28 + rng.below(14);
    for p in 0..paragraphs {
        let sentences = 2 + rng.below(3);
        for _ in 0..sentences {
            let words = 8 + rng.below(8);
            for w in 0..words {
                let word = WORDS[rng.below(WORDS.len())];
                if w == 0 {
                    out.push_str(&title_case(word));
                } else {
                    out.push(' ');
                    out.push_str(word);
                }
            }
            out.push_str(". ");
        }
        let _ = p;
        out.push_str("\n\n");
    }
    let _ = chapter_index;
    out
}

fn lock_cache(
    cache: &Mutex<HashMap<ChapterId, String>>,
) -> std::sync::MutexGuard<'_, HashMap<ChapterId, String>> {
    cache.lock().unwrap_or_else(|p| p.into_inner())
}

/// A deterministic mock content catalog.
pub struct MockCatalog {
    novels: Vec<Novel>,
    meta: HashMap<NovelId, NovelMeta>,
    chapters: HashMap<NovelId, Vec<Chapter>>,
    content_cache: Mutex<HashMap<ChapterId, String>>,
}

impl MockCatalog {
    /// Build the demo catalog. Deterministic across runs (timestamps are
    /// relative to the current day at day granularity).
    pub fn demo() -> Self {
        let today = Utc::now()
            .date_naive()
            .and_hms_opt(12, 0, 0)
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
            .unwrap_or_else(Utc::now);

        let mut novels = Vec::with_capacity(NOVELS.len());
        let mut meta = HashMap::new();
        let mut chapters: HashMap<NovelId, Vec<Chapter>> = HashMap::new();

        for (i, spec) in NOVELS.iter().enumerate() {
            let (source_id, _, _) = SOURCES[spec.source];
            let url = format!(
                "https://{source_id}.example/novels/{}",
                spec.title.to_lowercase().replace(' ', "-")
            );
            let mut novel = Novel::new(spec.title, &url)
                .with_author(spec.author)
                .with_summary(spec.summary)
                .with_status(spec.status);
            for genre in spec.genres {
                novel = novel.with_tag(*genre);
            }
            novel.add_source_ref(PluginId(source_id.to_string()), url.clone());
            novel.added_at = today - Duration::days(spec.added_days_ago);
            novel.updated_at = today - Duration::days(spec.updated_days_ago);

            meta.insert(
                novel.id,
                NovelMeta {
                    alt_titles: spec.alt_titles.iter().map(|s| s.to_string()).collect(),
                    artist: spec.artist.map(str::to_string),
                },
            );

            // Chapters, newest published `updated_days_ago`, weekly cadence.
            let mut list = Vec::with_capacity(spec.chapters as usize);
            for index in 0..spec.chapters {
                let seed = blake3::hash(format!("{url}#chapter-{index}").as_bytes());
                let title_seed = Prng::from_seed(seed.as_bytes()).below(100);
                let title = if title_seed < 55 {
                    let mut rng = Prng::from_seed(seed.as_bytes());
                    let a = title_case(WORDS[rng.below(WORDS.len())]);
                    let b = WORDS[rng.below(WORDS.len())];
                    format!("Chapter {}: The {a} of {b}", index + 1)
                } else {
                    format!("Chapter {}", index + 1)
                };
                let published = today
                    - Duration::days(spec.updated_days_ago)
                    - Duration::days(((spec.chapters - 1 - index) * 3) as i64);
                let chapter = Chapter::new(novel.id, index, title, seed)
                    .with_url(format!("{url}/chapters/{}", index + 1))
                    .with_published_date(published);
                list.push(chapter);
            }
            chapters.insert(novel.id, list);
            novels.push(novel);

            // Stable ordering independent of spec order changes.
            let _ = i;
        }

        Self {
            novels,
            meta,
            chapters,
            content_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Presentation metadata for a novel (alt titles, artist).
    pub fn novel_meta(&self, id: &NovelId) -> Option<NovelMeta> {
        self.meta.get(id).cloned()
    }

    fn content_seed(&self, chapter: &ChapterId) -> Option<blake3::Hash> {
        self.chapters
            .values()
            .flat_map(|list| list.iter())
            .find(|c| c.id == *chapter)
            .map(|c| c.content_hash)
    }
}

impl ContentRepository for MockCatalog {
    fn sources(&self) -> Vec<SourceInfo> {
        SOURCES
            .iter()
            .map(|(id, name, version)| SourceInfo {
                id: PluginId(id.to_string()),
                name: name.to_string(),
                version: version.to_string(),
                enabled: true,
                novel_count: self
                    .novels
                    .iter()
                    .filter(|n| n.source_refs.iter().any(|r| r.plugin_id.0 == *id))
                    .count(),
            })
            .collect()
    }

    fn all_novels(&self) -> Vec<Novel> {
        self.novels.clone()
    }

    fn novel(&self, id: &NovelId) -> Option<Novel> {
        self.novels.iter().find(|n| &n.id == id).cloned()
    }

    fn chapters(&self, novel: &NovelId) -> Vec<Chapter> {
        self.chapters.get(novel).cloned().unwrap_or_default()
    }

    fn chapter_content(&self, chapter: &ChapterId) -> Option<String> {
        if let Some(cached) = lock_cache(&self.content_cache).get(chapter) {
            return Some(cached.clone());
        }
        let seed = self.content_seed(chapter)?;
        let content = generate_content(seed.as_bytes(), 0);
        lock_cache(&self.content_cache).insert(*chapter, content.clone());
        Some(content)
    }

    fn chapter_size(&self, chapter: &ChapterId) -> u64 {
        let Some(seed) = self.content_seed(chapter) else {
            return 0;
        };
        let mut rng = Prng::from_seed(seed.as_bytes());
        14_000 + rng.next() % 12_000
    }

    fn search(&self, query: &str) -> Vec<NovelId> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return self.recently_updated(usize::MAX);
        }
        let mut ranked: Vec<(u8, String, NovelId)> = self
            .novels
            .iter()
            .filter_map(|novel| {
                let title = novel.title.to_lowercase();
                let rank = if title.contains(&query) {
                    Some(0)
                } else if self.meta.get(&novel.id).is_some_and(|m| {
                    m.alt_titles
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query))
                }) {
                    Some(1)
                } else if novel
                    .authors
                    .iter()
                    .any(|a| a.name.to_lowercase().contains(&query))
                {
                    Some(2)
                } else if novel
                    .tags
                    .iter()
                    .any(|t| t.name.to_lowercase().contains(&query))
                {
                    Some(3)
                } else {
                    None
                }?;
                Some((rank, title, novel.id))
            })
            .collect();
        ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        ranked.into_iter().map(|(_, _, id)| id).collect()
    }

    fn trending(&self, limit: usize) -> Vec<NovelId> {
        // Deterministic "popularity": hash-derived base + chapter count.
        let mut scored: Vec<(u64, &Novel)> = self
            .novels
            .iter()
            .map(|n| {
                let base = n.id.as_bytes()[0] as u64;
                let chapters = self.chapters.get(&n.id).map_or(0, Vec::len) as u64;
                (base.wrapping_mul(31) % 97 + chapters, n)
            })
            .collect();
        scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
        scored.into_iter().take(limit).map(|(_, n)| n.id).collect()
    }

    fn featured(&self, limit: usize) -> Vec<NovelId> {
        self.novels
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 3 == 0)
            .take(limit)
            .map(|(_, n)| n.id)
            .collect()
    }

    fn recently_added(&self, limit: usize) -> Vec<NovelId> {
        let mut novels: Vec<&Novel> = self.novels.iter().collect();
        novels.sort_by_key(|n| std::cmp::Reverse(n.added_at));
        novels.into_iter().take(limit).map(|n| n.id).collect()
    }

    fn recently_updated(&self, limit: usize) -> Vec<NovelId> {
        let mut novels: Vec<&Novel> = self.novels.iter().collect();
        novels.sort_by_key(|n| std::cmp::Reverse(n.updated_at));
        novels.into_iter().take(limit).map(|n| n.id).collect()
    }

    fn genres(&self) -> Vec<String> {
        let mut genres: Vec<String> = self
            .novels
            .iter()
            .flat_map(|n| n.tags.iter().map(|t| t.name.clone()))
            .collect();
        genres.sort();
        genres.dedup();
        genres
    }

    fn novels_by_genre(&self, genre: &str) -> Vec<NovelId> {
        self.novels
            .iter()
            .filter(|n| n.tags.iter().any(|t| t.name.eq_ignore_ascii_case(genre)))
            .map(|n| n.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::ContentRepository;

    #[test]
    fn demo_catalog_is_populated() {
        let catalog = MockCatalog::demo();
        assert_eq!(catalog.all_novels().len(), NOVELS.len());
        assert_eq!(catalog.sources().len(), 3);
        assert!(!catalog.genres().is_empty());
    }

    #[test]
    fn every_novel_has_chapters_and_content() {
        let catalog = MockCatalog::demo();
        for novel in catalog.all_novels() {
            let chapters = catalog.chapters(&novel.id);
            assert!(!chapters.is_empty(), "{} has no chapters", novel.title);
            let first = &chapters[0];
            let content = catalog.chapter_content(&first.id).expect("content");
            assert!(content.len() > 4000, "chapter content too small");
            assert!(catalog.chapter_size(&first.id) > 0);
        }
    }

    #[test]
    fn content_generation_is_deterministic() {
        let a = MockCatalog::demo();
        let b = MockCatalog::demo();
        let novel = a.all_novels()[0].id;
        let ch = a.chapters(&novel)[0].id;
        assert_eq!(a.chapter_content(&ch), b.chapter_content(&ch));
    }

    #[test]
    fn search_matches_title_author_and_tag() {
        let catalog = MockCatalog::demo();
        assert!(!catalog.search("moonlit").is_empty());
        assert!(!catalog.search("Kazehara").is_empty());
        assert!(!catalog.search("fantasy").is_empty());
        assert!(catalog.search("qqq-no-hit").is_empty());
    }

    #[test]
    fn empty_search_returns_all() {
        let catalog = MockCatalog::demo();
        assert_eq!(catalog.search("").len(), NOVELS.len());
    }

    #[test]
    fn trending_and_featured_are_bounded() {
        let catalog = MockCatalog::demo();
        assert_eq!(catalog.trending(5).len(), 5);
        assert_eq!(catalog.featured(4).len(), 4);
    }

    #[test]
    fn recently_lists_are_ordered() {
        let catalog = MockCatalog::demo();
        let added = catalog.recently_added(usize::MAX);
        for w in added.windows(2) {
            let a = catalog.novel(&w[0]).unwrap();
            let b = catalog.novel(&w[1]).unwrap();
            assert!(a.added_at >= b.added_at);
        }
    }

    #[test]
    fn genre_lookup_roundtrips() {
        let catalog = MockCatalog::demo();
        for genre in catalog.genres() {
            assert!(!catalog.novels_by_genre(&genre).is_empty());
        }
    }
}
