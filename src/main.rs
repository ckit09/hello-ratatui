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

// Configuration for each coin
#[derive(Debug, Clone)]
struct CoinConfig {
    symbol: String,
    display_name: String,
    color: Color,
    precision: usize,
}

impl Default for CoinConfig {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            display_name: String::new(),
            color: Color::White,
            precision: 2,
        }
    }
}

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
    prices: Vec<f64>,
    coin_configs: Vec<CoinConfig>,
    last_update: Instant,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        // Define your coin configurations here
        let coin_configs = vec![
            CoinConfig {
                symbol: "BTCUSDT".to_string(),
                display_name: "BTC/USDT".to_string(),
                color: Color::Green,
                precision: 2,
            },
            CoinConfig {
                symbol: "ETHUSDT".to_string(),
                display_name: "ETH/USDT".to_string(),
                color: Color::Blue,
                precision: 2,
            },
            CoinConfig {
                symbol: "BNBUSDT".to_string(),
                display_name: "BNB/USDT".to_string(),
                color: Color::Yellow,
                precision: 2,
            },
            CoinConfig {
                symbol: "UNIUSDT".to_string(),
                display_name: "UNI/USDT".to_string(),
                color: Color::Cyan,
                precision: 2,
            },
            CoinConfig {
                symbol: "TONUSDT".to_string(),
                display_name: "TON/USDT".to_string(),
                color: Color::Cyan,
                precision: 2,
            },
            CoinConfig {
                symbol: "SOLUSDT".to_string(),
                display_name: "SOL/USDT".to_string(),
                color: Color::Cyan,
                precision: 2,
            },
            CoinConfig {
                symbol: "XRPUSDT".to_string(),
                display_name: "XRP/USDT".to_string(),
                color: Color::Magenta,
                precision: 4,
            },
            CoinConfig {
                symbol: "DOGEUSDT".to_string(),
                display_name: "DOGE/USDT".to_string(),
                color: Color::LightYellow,
                precision: 6,
            },
            CoinConfig {
                symbol: "TONUSDT".to_string(),
                display_name: "TON/USDT".to_string(),
                color: Color::LightBlue,
                precision: 4,
            },
            CoinConfig {
                symbol: "ADAUSDT".to_string(),
                display_name: "ADA/USDT".to_string(),
                color: Color::LightCyan,
                precision: 4,
            },
        ];

        Self {
            running: false,
            prices: vec![0.0; coin_configs.len()],
            coin_configs,
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
                Constraint::Length((self.coin_configs.len() as u16) * 2 + 2), // Prices
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
        let prices_text: Vec<Line> = self.coin_configs.iter().enumerate().map(|(i, config)| {
            Line::from(vec![
                Span::raw(format!("{}:  ", config.display_name)),
                Span::styled(
                    format!("${:.prec$}", self.prices[i], prec = config.precision),
                    Style::default().fg(config.color),
                ),
            ])
        }).collect();

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
        for (i, config) in self.coin_configs.iter().enumerate() {
            if let Ok(price) = market.get_price(&config.symbol) {
                self.prices[i] = price.price;
            }
        }
        
        Ok(())
    }
}
