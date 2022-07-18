use tui::{
    backend::Backend,
    layout::{Rect, *},
    style::*,
    text::*,
    widgets::{Block, Borders, *},
    Frame,
};

pub(super) fn mk_text_line_fg(fg_color: Color, text: &str) -> Paragraph<'_> {
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(fg_color).bg(Color::Black))
        .wrap(Wrap { trim: true })
}

pub(super) fn render_border<B: Backend>(f: &mut Frame<B>, chunk: Rect, title: &str) -> Rect {
    f.render_widget(
        Block::default()
            .title(title)
            .style(Style::default().bg(Color::Black))
            .borders(Borders::ALL),
        chunk,
    );
    chunk.inner(&Margin {
        horizontal: 1,
        vertical: 1,
    })
}

pub(super) fn render_text<B: Backend>(f: &mut Frame<B>, chunk: Rect, text: &str) {
    let w = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });

    f.render_widget(w, chunk);
}

pub(super) fn render_left_text<B: Backend>(f: &mut Frame<B>, chunk: Rect, text: &str) {
    let w = Paragraph::new(text)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });

    f.render_widget(w, chunk);
}

pub(super) fn rect_to_lines(r: Rect) -> Vec<Rect> {
    (0..r.height)
        .map(|i| Rect {
            x: r.x,
            y: r.y + i,
            height: 1,
            width: r.width,
        })
        .collect()
}

pub(super) fn mk_button(label: &str, highlight: bool, can_afford: bool) -> Paragraph<'static> {
    let color = if highlight {
        Color::Yellow
    } else {
        Color::White
    };
    let modifier = if can_afford {
        Modifier::BOLD | Modifier::UNDERLINED
    } else {
        Modifier::empty()
    };
    let text = Span::styled(
        format!("  {label}  "),
        Style::default().fg(color).add_modifier(modifier),
    );
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
}

pub(super) fn mk_button_align(
    label: &str,
    highlight: bool,
    can_afford: bool,
    alignment: Alignment,
) -> Paragraph<'static> {
    let color = if highlight {
        Color::Yellow
    } else {
        Color::White
    };
    let modifier = if can_afford {
        Modifier::BOLD | Modifier::UNDERLINED
    } else {
        Modifier::empty()
    };
    let text = Span::styled(
        format!("  {label}  "),
        Style::default().fg(color).add_modifier(modifier),
    );
    Paragraph::new(text)
        .alignment(alignment)
        .wrap(Wrap { trim: true })
}
