use failure::Fallible;

mod imp;
mod ui;

fn main() -> Fallible<()> {
    ui::main()
}
