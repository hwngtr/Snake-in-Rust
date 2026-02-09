use rand::Rng;
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Plain,
    Snake,
    Wall,
    Food,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

pub struct Snake {
    pub coordinates: VecDeque<usize>,
    pub direction: Direction,
}

pub struct Game {
    pub cells: Vec<Cell>,
    pub width: usize,
    pub height: usize,
    pub snake: Snake,
    pub score: u32,
    pub game_over: bool,
    pub grow_on_eat: bool,
}

impl Game {
    pub fn new(width: usize, height: usize, grow_on_eat: bool) -> Self {
        let mut cells = vec![Cell::Plain; width * height];

        for x in 0..width {
            cells[x] = Cell::Wall;
            cells[x + (height - 1) * width] = Cell::Wall;
        }
        for y in 0..height {
            cells[y * width] = Cell::Wall;
            cells[y * width + width - 1] = Cell::Wall;
        }

        let snake_pos = 2 * width + 2;
        let mut snake_coords = VecDeque::new();
        snake_coords.push_front(snake_pos);
        cells[snake_pos] = Cell::Snake;

        let mut game = Game {
            cells,
            width,
            height,
            snake: Snake {
                coordinates: snake_coords,
                direction: Direction::Right,
            },
            score: 0,
            game_over: false,
            grow_on_eat,
        };
        game.place_food();
        game
    }

    pub fn from_string(compressed: &str, grow_on_eat: bool) -> Result<Self, String> {
        let parts: Vec<&str> = compressed.split('|').collect();
        if parts.is_empty() {
            return Err("Empty string".to_string());
        }

        // Parse Dimensions B<height>x<width>
        let dims = parts[0];
        if !dims.starts_with('B') {
            return Err("Invalid dimension format".to_string());
        }
        let dims_parts: Vec<&str> = dims[1..].split('x').collect();
        if dims_parts.len() != 2 {
            return Err("Invalid dimension format".to_string());
        }
        let height: usize = dims_parts[0].parse().map_err(|_| "Invalid height")?;
        let width: usize = dims_parts[1].parse().map_err(|_| "Invalid width")?;

        let mut cells = Vec::new();
        let mut snake_coords = VecDeque::new();
        let mut snake_found = false;

        for row_str in parts.iter().skip(1) {
            let mut chars = row_str.chars().peekable();
            while let Some(c) = chars.next() {
                if !c.is_alphabetic() {
                    continue;
                }

                let mut num_str = String::new();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_digit(10) {
                        num_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                let count: usize = num_str.parse().unwrap_or(0);

                let cell_type = match c {
                    'W' => Cell::Wall,
                    'E' => Cell::Plain,
                    'S' => Cell::Snake,
                    _ => return Err(format!("Unknown char {}", c)),
                };

                for _ in 0..count {
                    if cell_type == Cell::Snake {
                        if snake_found {
                            return Err("Multiple snakes".to_string());
                        }
                        snake_coords.push_front(cells.len());
                        snake_found = true;
                    }
                    cells.push(cell_type);
                }
            }
        }

        if cells.len() != width * height {
            return Err(format!(
                "Dimension mismatch: expected {}, got {}",
                width * height,
                cells.len()
            ));
        }
        if !snake_found {
            return Err("No snake found".to_string());
        }

        let mut game = Game {
            cells,
            width,
            height,
            snake: Snake {
                coordinates: snake_coords,
                direction: Direction::Right,
            },
            score: 0,
            game_over: false,
            grow_on_eat,
        };
        game.place_food();
        Ok(game)
    }

    pub fn place_food(&mut self) {
        let mut rng = rand::thread_rng();
        loop {
            let idx = rng.gen_range(0..self.cells.len());
            if self.cells[idx] == Cell::Plain {
                self.cells[idx] = Cell::Food;
                return;
            }
        }
    }

    pub fn update(&mut self, input: Direction) {
        if self.game_over {
            return;
        }

        let mut new_dir = input;

        if self.snake.coordinates.len() > 1 {
            match (self.snake.direction, input) {
                (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left) => {
                    new_dir = self.snake.direction;
                }
                (Direction::None, _) => {
                    new_dir = input;
                }
                (_, Direction::None) => {
                    new_dir = self.snake.direction;
                }
                _ => {}
            }
        }

        if new_dir == Direction::None {
            new_dir = self.snake.direction;
        }

        self.snake.direction = new_dir;

        let head_idx = *self.snake.coordinates.front().unwrap();
        let head_x = head_idx % self.width;
        let head_y = head_idx / self.width;

        let (next_x, next_y) = match self.snake.direction {
            Direction::Up => (head_x, head_y.wrapping_sub(1)),
            Direction::Down => (head_x, head_y + 1),
            Direction::Left => (head_x.wrapping_sub(1), head_y),
            Direction::Right => (head_x + 1, head_y),
            Direction::None => (head_x, head_y),
        };

        if next_x >= self.width || next_y >= self.height {
            self.game_over = true;
            return;
        }

        let next_idx = next_y * self.width + next_x;

        match self.cells[next_idx] {
            Cell::Wall | Cell::Snake => {
                let tail_idx = *self.snake.coordinates.back().unwrap();
                if next_idx == tail_idx {
                } else {
                    self.game_over = true;
                    return;
                }
            }
            _ => {}
        }

        let is_food = self.cells[next_idx] == Cell::Food;

        self.snake.coordinates.push_front(next_idx);
        self.cells[next_idx] = Cell::Snake;

        if is_food {
            self.score += 1;
            if self.grow_on_eat {
                self.place_food();
            } else {
                let tail = self.snake.coordinates.pop_back().unwrap();
                self.cells[tail] = Cell::Plain;
                self.cells[next_idx] = Cell::Snake;
                self.place_food();
            }
        } else {
            let tail = self.snake.coordinates.pop_back().unwrap();
            self.cells[tail] = Cell::Plain;
            self.cells[next_idx] = Cell::Snake;
        }
    }
}
