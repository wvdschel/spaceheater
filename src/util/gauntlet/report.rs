use std::{
    fs,
    io::{self, Write},
    time::SystemTime,
};

pub struct Score {
    pub snake_name: String,
    pub snake_config: String,
    pub points: isize,
}

pub fn write_report(report_name: &str, scores: &Vec<Score>) -> io::Result<()> {
    let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let filename = format!("{}_{}.txt", report_name, timestamp);
    let mut file = fs::File::create(filename)?;
    for (rank, score) in scores.iter().enumerate() {
        file.write(
            format!(
                "#{}: {} with {} points ({})\n",
                rank, score.snake_name, score.points, score.snake_config
            )
            .as_bytes(),
        )?;
    }
    Ok(())
}
