use std::{
    fs,
    io::{self, Write},
};

use super::Score;

pub fn write_report(report_name: &str, scores: &Vec<Score>) -> io::Result<()> {
    let filename = format!("{}.txt", report_name);
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
