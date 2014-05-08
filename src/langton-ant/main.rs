#![feature(phase)]

extern crate getopts;
extern crate sdl;
extern crate rand;
extern crate regex;
#[phase(syntax)] extern crate regex_macros;

static UP: int = 0;
static RIGHT: int = 1;
static DOWN: int = 2;
static LEFT: int = 3;

static WHITE: int = 0;
static BLACK: int = 1;

static VIEW_MOVE_STEP: int = 5;
static VIEW_ZOOM_GX_PLUS: f32 = 1.1;
static VIEW_ZOOM_GX_MINUS: f32 = 0.9;

struct Ant {
    x: int,
    y: int,
    dir: int,
}

struct World {
    // current iteration
    it: uint,

    // size of the map
    width: int,
    height: int,

    // the map
    tab: Vec<Vec<int>>,

    // the ant
    ant: Ant,

    // window resolution
    screen_width: int,
    screen_height: int,

    // SDL surface
    screen: ~sdl::video::Surface,

    // interval between cycles and screen refreshing
    refresh_interval: uint,
    cycle_interval: uint,

    // position of the camera
    view_pos_x: int,
    view_pos_y: int,

    // option camera follows the ant
    follow_ant: bool,

    // size in pixel of a square
    square_size: int,

    // number of squares per line and columns
    squares_per_line: int,
    squares_per_column: int,
}

impl Ant {

    pub fn turn_left(&mut self) {
        self.dir = (self.dir - 1 + 4) % 4;
    }

    pub fn turn_right(&mut self) {
        self.dir = (self.dir + 1) % 4;
    }
}

impl World {

    pub fn new(width: uint, height: uint, screen_width: uint, screen_height: uint,
     refresh_interval: uint, cycle_interval: uint) -> Option<World> {
        use rand::random;

        if width > 0 && height > 0 {
            sdl::init([sdl::InitVideo]);
            std::rt::at_exit(sdl::quit);
            sdl::wm::set_caption("Langton's ant", "");

            let mut world  = World {
                it: 0,
                width: width as int,
                height: height as int,
                tab: Vec::from_elem(height, Vec::from_elem(width, 0)),
                ant: Ant {
                    x: (random::<uint>() % width) as int,
                    y: (random::<uint>() % height) as int,
                    dir: (random::<uint>() % 4) as int,
                },
                screen_width: screen_width as int,
                screen_height: screen_height as int,
                screen: World::set_video(screen_width as int, screen_height as int),
                refresh_interval: refresh_interval,
                cycle_interval: cycle_interval,
                view_pos_x:  width as int / 2,
                view_pos_y: height as int / 2,
                follow_ant: true,
                square_size: 0,
                squares_per_line: 0,
                squares_per_column: 0,
            };
            world.fit();
            world.update_scale();
            Some(world)
        }
        else {
            None
        }
    }

    // launch the game
    pub fn run(&mut self) {

        World::print_help();

        let mut refresh_ticks: uint = 0;
        let mut cycle_ticks: uint = 0;
        let mut move_ticks: uint = 0;
        let mut move_buttons: [bool, ..4] = [false, ..4];

        self.draw_world();
        if self.follow_ant {
            self.set_view_above_ant();
        }
        self.refresh_screen();

        loop {
            let current_ticks: uint = sdl::get_ticks();

            // manage events
            match sdl::event::poll_event() {
                sdl::event::QuitEvent => break,
                sdl::event::KeyEvent(k, t, _, _) if t => match k {
                    sdl::event::EscapeKey => break,
                    sdl::event::KpPlusKey => self.zoom(VIEW_ZOOM_GX_PLUS),
                    sdl::event::KpMinusKey => self.zoom(VIEW_ZOOM_GX_MINUS),
                    sdl::event::UpKey => move_buttons[UP as uint] = true,
                    sdl::event::DownKey => move_buttons[DOWN as uint] = true,
                    sdl::event::LeftKey => move_buttons[LEFT as uint] = true,
                    sdl::event::RightKey => move_buttons[RIGHT as uint] = true,
                    sdl::event::FKey => self.follow_ant ^= true,
                    _ => {}
                },
                sdl::event::KeyEvent(k, t, _, _) if !t => match k {
                    sdl::event::UpKey => move_buttons[UP as uint] = false,
                    sdl::event::DownKey => move_buttons[DOWN as uint] = false,
                    sdl::event::LeftKey => move_buttons[LEFT as uint] = false,
                    sdl::event::RightKey => move_buttons[RIGHT as uint] = false,
                    _ => {}
                },
                _ => {},
            };

            // aplly view movements if the ant isn't followed
            if !self.follow_ant {
                if current_ticks - move_ticks > 20 {
                    move_ticks = current_ticks;
                    for dir in range(0, 4) {
                        if move_buttons[dir as uint] {
                            self.move_view(dir);
                        }
                    }
                }
            }

            // game
            if current_ticks - cycle_ticks >= self.cycle_interval {
                cycle_ticks = current_ticks;
                self.do_one_cycle();
                self.it += 1;
            }

            // refresh screen, or not.
            if current_ticks - refresh_ticks >= self.refresh_interval {
                refresh_ticks = current_ticks;
                if self.follow_ant {
                    self.set_view_above_ant();
                }
                self.draw_world();
                self.refresh_screen();
                self.print_infos();
            }
        }
        println!("");
    }

    // do one cyle of the game
    fn do_one_cycle(&mut self) {
        match self.get(self.ant.x, self.ant.y) {
            WHITE => self.ant.turn_right(),
            BLACK => self.ant.turn_left(),
            _ => unreachable!(),
        };
        self.inverse(self.ant.x, self.ant.y);
        let (x, y) =
        match self.ant.dir {
            UP => (0, -1),
            RIGHT => (1, 0),
            DOWN => (0, 1),
            LEFT => (-1, 0),
            _ => unreachable!(),
        };
        self.move_ant(x, y);
    }

    // return a new SDL screen surface
    fn set_video(width: int, height: int) -> ~sdl::video::Surface {
        match sdl::video::set_video_mode(width, height, 32,
           [sdl::video::HWSurface],
           [sdl::video::DoubleBuf]) {
            Ok(screen) => screen,
            Err(err) => fail!("failed to set video mode: {}", err)
        }
    }

    // set a square of the map
    fn set(&mut self, x: int, y: int, value: int) {
        *self.tab
        .get_mut(((y + self.height) % self.height) as uint)
        .get_mut(((x + self.width) % self.width) as uint) = value;
    }

    // get a square of the map
    fn get(&self, x: int, y: int) -> int {
        *self.tab
        .get(((y + self.height) % self.height) as uint)
        .get(((x + self.width) % self.width) as uint)
    }

    // inverse the value of a square of the map
    fn inverse(&mut self, x: int, y: int) {
        let old = self.get(x, y);
        self.set(x, y, old ^ 1);
    }

    // move the ant to current + (x,y)
    fn move_ant(&mut self, x: int, y: int) {
        self.ant.x = (self.ant.x + x + self.width) % self.width;
        self.ant.y = (self.ant.y + y + self.height) % self.height;
    }

    // set the square_size to the best value
    fn fit(&mut self) {
        self.square_size =
        std::cmp::max(self.screen_width / self.width,
          self.screen_height / self.height);
        if self.square_size < 1 {
            self.square_size = 1;
        }
    }

    // re-center the camera if it is out the map.
    fn adjust_view_position(&mut self) {
        if self.view_pos_x < self.squares_per_line / 2 {
            self.view_pos_x = self.squares_per_line / 2;
        }
        if self.view_pos_x > self.width - self.squares_per_line / 2 {
            self.view_pos_x = self.width - self.squares_per_line / 2;
        }
        if self.view_pos_y < self.squares_per_column / 2 {
            self.view_pos_y = self.squares_per_column / 2;
        }
        if self.view_pos_y > self.height - self.squares_per_column / 2 {
            self.view_pos_y = self.height - self.squares_per_column / 2;
        }
    }

    // update some members after changing square_size
    fn update_scale(&mut self) {
        self.squares_per_line = std::cmp::min(self.screen_width / self.square_size, self.width);
        self.squares_per_column = std::cmp::min(self.screen_height / self.square_size, self.height);
        self.adjust_view_position();
    }

    // draw a square of the map at a screen position
    fn draw_square(&self, map_x: int, map_y: int,
      scr_x: int, scr_y: int) {
        let rect = sdl::Rect {
            x: scr_x as i16,
            y: scr_y as i16,
            w: self.square_size as u16,
            h: self.square_size as u16,
        };
        let color: sdl::video::Color =
        if self.ant.x == map_x && self.ant.y == map_y {
            sdl::video::RGB(255, 0, 0)
        } else {
            match self.get(map_x, map_y) {
                WHITE => sdl::video::RGB(20, 20, 20),
                BLACK => sdl::video::RGB(0, 150, 50),
                _=> unreachable!(),
            }
        };
        self.screen.fill_rect(Some(rect), color);
    }

    // draw all the map
    fn draw_world(&self) {

        self.screen.clear();

        let first_visible_x = self.view_pos_x - self.squares_per_line / 2;
        let first_visible_y = self.view_pos_y - self.squares_per_column / 2;

        let first_screen_x = self.screen_width / 2 - (self.squares_per_line * self.square_size) / 2;
        let first_screen_y = self.screen_height / 2 - (self.squares_per_column * self.square_size) / 2;

        for x in range(0, self.squares_per_line) {
            if first_visible_x + x >= self.width {
                break;
            }
            else if first_visible_x + x < 0 {
                continue;
            }
            for y in range(0, self.squares_per_column) {
                if first_visible_y + y >= self.height {
                    break;
                }
                else if first_visible_y + y < 0 {
                    continue;
                }
                self.draw_square(first_visible_x + x, first_visible_y + y,
                   first_screen_x + x * self.square_size, first_screen_y + y * self.square_size);
            }
        }
    }

    // set the camera above the ant
    fn set_view_above_ant(&mut self) {
        if self.ant.x < self.view_pos_x - self.squares_per_line / 4
        || self.ant.y < self.view_pos_y - self.squares_per_column / 4
        || self.ant.x > self.view_pos_x + self.squares_per_line / 4
        || self.ant.y > self.view_pos_y + self.squares_per_column / 4 {
            self.view_pos_x = self.ant.x;
            self.view_pos_y = self.ant.y;
            self.adjust_view_position();
        }
    }

    // apply graphic modifications to the screen
    fn refresh_screen(&mut self) {
        self.screen.flip();
    }

    // zoom plus or minus
    fn zoom(&mut self, gx: f32)
    {
        assert!(gx > 0.0);
        if (gx > 1.0 && self.square_size <
            std::cmp::min(self.screen_width, self.screen_height)) ||
        (gx < 1.0 && self.square_size > 1) {
            let new_sq_size = (self.square_size as f32 * gx) as int;
            if new_sq_size == self.square_size {
                if gx > 1.0 {
                    self.square_size += 1;
                }
                else if gx < 1.0 {
                    self.square_size -= 1;
                }
            }
            else {
                self.square_size = new_sq_size;
            }
            self.update_scale();
            self.draw_world();
            self.refresh_screen();
        }
    }

    fn move_view_up(&mut self) {
        self.view_pos_y = std::cmp::max(self.squares_per_column / 2, self.view_pos_y - VIEW_MOVE_STEP);
    }

    fn move_view_down(&mut self) {
        self.view_pos_y = std::cmp::min(self.height - self.squares_per_column / 2,
            self.view_pos_y + VIEW_MOVE_STEP);
    }

    fn move_view_left(&mut self) {
        self.view_pos_x = std::cmp::max(self.squares_per_line / 2, self.view_pos_x - VIEW_MOVE_STEP);
    }

    fn move_view_right(&mut self) {
        self.view_pos_x = std::cmp::min(self.width - self.squares_per_line / 2,
            self.view_pos_x + VIEW_MOVE_STEP);
    }

    // move camera in the specified direction
    fn move_view(&mut self, direction: int) {

        if self.follow_ant {
            return ;
        }

        match direction {
            UP => self.move_view_up(),
            DOWN => self.move_view_down(),
            LEFT => self.move_view_left(),
            RIGHT =>self.move_view_right(),
            _ => unreachable!(),
        };
        self.draw_world();
        self.refresh_screen();
    }

    // print world information on stdout
    fn print_infos(&self) {
        print!("\r=> iteration: {0} | ant: ({1},{2}) | camera: ({3},{4}) | \
            refresh_interval: {5}ms | cycle_interval: {6}ms",
            self.it, self.ant.x, self.ant.y, self.view_pos_x, self.view_pos_y,
            self.refresh_interval, self.cycle_interval);
        ::std::io::stdio::flush();
    }

    // print keyboard control help on sdtout
    fn print_help() {
        std::io::stdio::println("##################### keyboard control #####################");
        std::io::stdio::println("#                                                          #");
        std::io::stdio::println("#        [+]       : zoom +                                #");
        std::io::stdio::println("#        [-]       : zoom -                                #");
        std::io::stdio::println("#        [F]       : enable/disable follow-ant mode        #");
        std::io::stdio::println("#  [^],[v],[<],[>] : move camera (if follow-ant disable)   #");
        std::io::stdio::println("#       [ESC]      : exit                                  #");
        std::io::stdio::println("#                                                          #");
        std::io::stdio::println("############################################################");
        println!("");
    }
}


struct Arguments {
    map_width: uint,
    map_height: uint,  
    window_width: uint,
    window_height: uint,
    refresh_interval: uint,
    cycle_interval: uint,
}

impl Arguments {

    // arguments parsing
    pub fn new() -> Option<Arguments> {
        use getopts::{optopt, optflag, getopts, usage};

        let argv: ~[~str] = std::os::args();

        let opts = [
            optopt("m", "map-size", "the map size", "WIDTHxHEIGHT"),
            optopt("w", "window-size", "the window resolution", "WIDTHxHEIGHT"),
            optopt("r", "refresh", "interval between each refresh (ms)", "TIME"),
            optopt("c", "cycle", "interval between each cycle (ms)", "TIME"),
            optflag("h", "help", "display this help"),
        ];

        let matches = match getopts(argv.tail(), opts) {
            Ok(m) => m,
            Err(f) => {
                println!("{}\nTry -h or --help to see usage.", f.to_err_msg());
                return None;
            },
        };

        if matches.opt_present("h") {
            println!("Usage: ./main [options]\n{}", usage("Langton's Ant rust implementation.", opts));
            return None; 
        };

        let (map_width, map_height) = match matches.opt_str("m") {
            Some(s) => {
                let caps = match regex!(r"^(?P<width>\d+)x(?P<height>\d+)$").captures(s) {
                    Some(c) => c,
                    None => {
                            println!("invalid map size syntax: expected in the format WIDTHxHEIGHT");
                            return None;
                        },
                };
                (from_str(caps.name("width")).unwrap(), from_str(caps.name("height")).unwrap())
            }
            None => (500, 500),
        };

       let (window_width, window_height) = match matches.opt_str("w") {
            Some(s) => {
                let caps = match regex!(r"^(?P<width>\d+)x(?P<height>\d+)$").captures(s) {
                    Some(c) => c,
                    None => {
                            println!("invalid window resolution syntax: expected in the format WIDTHxHEIGHT");
                            return None;
                        },
                };
                (from_str(caps.name("width")).unwrap(), from_str(caps.name("height")).unwrap())
            }
            None => (800, 600),
        };

        let refresh_itv: uint = match matches.opt_str("r") {
            Some(s) => match from_str(s) {
                Some(x) => x,
                None => {
                    println!("error: bad value type for '-r': uint expected");
                    return None;
                },
            },
            None => 40,
        };

        let cycle_itv: uint = match matches.opt_str("c") {
            Some(s) => match from_str(s) {
                Some(x) => x,
                None => {
                    println!("error: bad value type for '-c': uint expected");
                    return None;
                },
            },
            None => 5,
        };

        return Some(Arguments {
            map_width: map_width,
            map_height: map_height,
            window_width: window_width,
            window_height: window_height,
            refresh_interval: refresh_itv,
            cycle_interval: cycle_itv,
        });
    }
}

fn main() {
    
    let args = match Arguments::new(){
            Some(a) => a,
            None => return,
        };

    World::new(args.map_width, args.map_height,
                args.window_width, args.window_height,
                args.refresh_interval, args.cycle_interval)
        .expect("map size could not be null")
        .run();
}