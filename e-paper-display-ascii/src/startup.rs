use ratatui::{
    Terminal,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    widgets::Paragraph,
};

const LOGO: &str = r#"
 ██▒   █▓  ██████    ▄████ 
▓██░   █▒▒██    ▒ ▒ ██▒ ▀█▒
 ▓██  █▒░░ ▓██▄   ░▒██░▄▄▄░
  ▒██ █░░  ▒   ██▒░░▓█  ██▓
   ▒▀█░  ▒██████▒▒░▒▓███▀▒░
   ░ ▐░  ▒ ▒▓▒ ▒ ░ ░▒   ▒  
   ░ ░░  ░ ░▒  ░    ░   ░  
     ░░  ░  ░  ░  ░ ░   ░ ░
      ░        ░        ░  
"#;

pub struct Startup;

impl Startup {
    pub fn start<B>(terminal: &mut Terminal<B>)
    where
        B: ratatui::backend::Backend,
    {
        terminal
            .draw(|frame| {
                let logo_widget = Paragraph::new(LOGO)
                    .style(Style::default())
                    .alignment(Alignment::Center)
                    .block(
                        ratatui::widgets::Block::default()
                            .style(Style::default().bg(Color::White).fg(Color::Black)),
                    );

                frame.render_widget(logo_widget, frame.area());
            })
            .unwrap();
    }
}
