use ember::agent::bdi::sensor::Percept;

#[derive(Percept)]
union Bad {
    x: f32,
    y: u32,
}

fn main() {}
