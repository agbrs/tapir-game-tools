use catppuccin_egui::Theme;

use egui::{
    emath::RectTransform, epaint::RectShape, Align2, FontId, Pos2, Rect, Rounding, Sense, Stroke,
    Ui, Vec2,
};

use super::{piano, Note};

const TIMELINE_ITEM_WIDTH: f32 = piano::PIANO_KEY_HEIGHT;

const TIMELINE_TOTAL_HEIGHT: f32 = piano::PIANO_KEY_HEIGHT * piano::NUM_WHITE_KEYS as f32;

pub struct TimelineSettings {
    pub beats_per_bar: usize,
}

pub struct TimelineResponse {
    pub hovered_beat_note: Option<(usize, piano::Note)>,
}

pub fn timeline(ui: &mut Ui, theme: &Theme, settings: TimelineSettings) -> TimelineResponse {
    let num_timeline_items = 0x40;

    let (response, painter) = ui.allocate_painter(
        Vec2::new(
            TIMELINE_ITEM_WIDTH * num_timeline_items as f32,
            TIMELINE_TOTAL_HEIGHT,
        ),
        Sense::hover(),
    );

    let to_screen = RectTransform::from_to(
        Rect::from_min_size(Pos2::ZERO, response.rect.size()),
        response.rect,
    );

    let timeline_rect = Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(TIMELINE_ITEM_WIDTH, TIMELINE_TOTAL_HEIGHT),
    );

    let hovered_beat_note = response.hover_pos().map(|hovered_pos| {
        let hovered_local_pos = to_screen.inverse().transform_pos(hovered_pos);

        let hovered_note = Note::from_y(hovered_local_pos.y);

        let hovered_beat = (hovered_local_pos.x / TIMELINE_ITEM_WIDTH).floor() as usize;

        (hovered_beat, hovered_note)
    });

    // render the vertical bars
    for beat in 0..num_timeline_items {
        let is_highlighted = hovered_beat_note.map(|(beat, _)| beat) == Some(beat);

        let this_rect = to_screen.transform_rect(
            timeline_rect.translate(Vec2::new(beat as f32 * TIMELINE_ITEM_WIDTH, 0.)),
        );

        let is_first_for_bar = beat % settings.beats_per_bar == 0;
        let fill_colour = if is_highlighted {
            theme.surface2
        } else if is_first_for_bar {
            theme.surface0
        } else {
            theme.surface1
        };

        painter.add(RectShape::new(
            this_rect,
            Rounding::ZERO,
            fill_colour,
            Stroke::new(1., theme.base),
        ));

        if is_first_for_bar {
            let text_y_gap = piano::PIANO_KEY_HEIGHT * 7.;
            let x_pos = this_rect.min.x + this_rect.width() / 2.;

            for repeat in 0..=(timeline_rect.height() / text_y_gap) as i32 {
                painter.text(
                    Pos2::new(
                        x_pos,
                        text_y_gap * repeat as f32 + this_rect.min.y + 2. * piano::PIANO_KEY_HEIGHT,
                    ),
                    Align2::CENTER_TOP,
                    beat,
                    FontId::monospace(12.),
                    theme.text,
                );
            }
        }
    }

    TimelineResponse { hovered_beat_note }
}
