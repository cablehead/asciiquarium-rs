//! Parity checks for the assembled creatures: the tricky ones (shark, sea
//! monster, big fish) must render with no leftover '?' transparency glyphs, and
//! the whale must actually blow a spout above its body.

use asciiquarium::entity::Entity;
use asciiquarium::render::Screen;
use asciiquarium::spawn;

/// Render each frame of an entity onto its own blank screen, as plain text.
fn frames_text(mut e: Entity) -> Vec<String> {
    e.x = 5.0;
    e.y = 2.0;
    (0..e.frames.len())
        .map(|i| {
            e.frame = i as f64;
            let f = e.current();
            let mut s = Screen::new(140, 40);
            s.blit(
                e.x.round() as i32,
                e.y.round() as i32,
                &f.shape,
                &f.mask,
                e.default_color,
                e.auto_trans,
                e.trans,
            );
            s.to_text()
        })
        .collect()
}

fn assert_no_question_marks(entities: Vec<Entity>) {
    for e in entities {
        for (i, text) in frames_text(e).into_iter().enumerate() {
            assert!(
                !text.contains('?'),
                "frame {i} rendered a literal '?': it should be transparent\n{text}"
            );
        }
    }
}

#[test]
fn specials_have_no_stray_transparency_glyphs() {
    let mut rng = rand::thread_rng();
    // Run each a few times to cover both facings and the new/classic art.
    for _ in 0..8 {
        assert_no_question_marks(spawn::shark(200, 40, &mut rng));
        assert_no_question_marks(spawn::monster(200, 40, false, &mut rng));
        assert_no_question_marks(spawn::monster(200, 40, true, &mut rng));
        assert_no_question_marks(vec![spawn::big_fish(200, 40, false, &mut rng)]);
        assert_no_question_marks(vec![spawn::big_fish(200, 40, true, &mut rng)]);
        assert_no_question_marks(spawn::whale(200, 40, &mut rng));
    }
}

#[test]
fn fish_render_without_transparency_glyphs() {
    let mut rng = rand::thread_rng();
    for _ in 0..40 {
        assert_no_question_marks(vec![spawn::fish(200, 40, false, &mut rng)]);
    }
}

#[test]
fn whale_blows_a_spout_above_its_body() {
    let mut rng = rand::thread_rng();
    let whale = spawn::whale(200, 40, &mut rng).into_iter().next().unwrap();
    let texts = frames_text(whale);

    // The body (with its eye) is present in every frame ...
    assert!(
        texts.iter().all(|t| t.contains("(o)")),
        "whale body/eye missing from some frame"
    );
    // ... and the spout frames draw strictly more ink than the body-only ones
    // (the whale body itself contains a ':', so glyph-matching won't do).
    let ink: Vec<usize> = texts
        .iter()
        .map(|t| t.chars().filter(|c| !c.is_whitespace()).count())
        .collect();
    let (min, max) = (*ink.iter().min().unwrap(), *ink.iter().max().unwrap());
    assert!(
        max > min,
        "no frame drew a spout above the body (ink range {min}..={max})"
    );
}
