use std::{
    fs,
    io::{self, Write},
    time::SystemTime,
};

use super::Score;

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
                rank,
                score.snake_name,
                score.points,
                score
                    .snake_config
                    .map(|c| c.to_string())
                    .unwrap_or("reference snake".to_string())
            )
            .as_bytes(),
        )?;
    }
    Ok(())
}
