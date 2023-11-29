mod slidingpuzzle;
mod tictactoe;
mod ultimatetictactoe;
mod user;

pub use slidingpuzzle::SlidingPuzzle;
pub use slidingpuzzle::SlidingPuzzleFilters;
pub use slidingpuzzle::SlidingPuzzleWithUser;

pub use tictactoe::TTTLeaderboardEntry;
pub use tictactoe::TicTacToe;

pub use ultimatetictactoe::UTTTLeaderboardEntry;
pub use ultimatetictactoe::UltimateTicTacToe;

pub use user::User;

const POINTS_PER_WIN: f64 = 17.0;
const POINTS_PER_LOSS: f64 = -13.0;

fn calculate_rating(wins: i64, losses: i64) -> f64 {
    // we want to generate a rating based on total number of games played, wins, and losses. preferrably something between 0 and 1000.
    // we'll use a sigmoid function to generate the rating.
    // the sigmoid function is f(x) = 1 / (1 + e^(-x))

    // let x = wins - losses;

    // let rating = 1.0 / (1.0 + (-x as f64).exp());

    // rating * 1000.0

    (wins as f64 * POINTS_PER_WIN) + (losses as f64 * POINTS_PER_LOSS)
}
