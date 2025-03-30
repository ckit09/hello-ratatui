use binance::api::*;
use binance::market::Market;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
    prelude::Stylize,
};
use std::time::{Duration, Instant};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    prices: Prices,
    last_update: Instant,
}

#[derive(Debug, Default)]
struct Prices {
    btc: f64,
    ton: f64,
    aave: f64,
    xrp: f64,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: false,
            prices: Prices::default(),
            last_update: Instant::now(),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            
            // Update prices every second
            if self.last_update.elapsed() >= Duration::from_secs(1) {
                self.update_prices()?;
                self.last_update = Instant::now();
            }
            
            // Poll for events with a timeout to prevent CPU spinning
            if event::poll(Duration::from_millis(250))? {
                self.handle_crossterm_events()?;
            }
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(10), // Prices
            ])
            .split(frame.area());

        // Title
        let title = Line::from(vec![
            Span::styled(
                "Crypto Price Tracker",
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue)
            )
        ]).centered();
        frame.render_widget(
            Paragraph::new(title).block(Block::default().borders(Borders::ALL)),
            chunks[0],
        );

        // Prices
        let prices_text = vec![
            Line::from(vec![
                Span::raw("BTC/USDT:  "),
                Span::styled(
                    format!("${:.2}", self.prices.btc),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::raw("TON/USDT:  "),
                Span::styled(
                    format!("${:.4}", self.prices.ton),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("AAVE/USDT: "),
                Span::styled(
                    format!("${:.2}", self.prices.aave),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::raw("XRP/USDT:  "),
                Span::styled(
                    format!("${:.4}", self.prices.xrp),
                    Style::default().fg(Color::Magenta),
                ),
            ]),
        ];

        frame.render_widget(
            Paragraph::new(prices_text)
                .block(Block::default().borders(Borders::ALL).title("Live Prices"))
                .style(Style::default().fg(Color::White)),
            chunks[1],
        );
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    fn update_prices(&mut self) -> Result<()> {
        let market = Market::new(None, None);
        
        // Get prices for each symbol
        if let Ok(btc_price) = market.get_price("BTCUSDT") {
            self.prices.btc = btc_price.price;
        }
        if let Ok(ton_price) = market.get_price("TONUSDT") {
            self.prices.ton = ton_price.price;
        }
        if let Ok(aave_price) = market.get_price("AAVEUSDT") {
            self.prices.aave = aave_price.price;
        }
        if let Ok(xrp_price) = market.get_price("XRPUSDT") {
            self.prices.xrp = xrp_price.price;
        }
        
        Ok(())
    }
}
