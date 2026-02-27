use anyhow::Result;
use clouder::run;

#[cfg(test)]
mod tests;

fn main() -> Result<()> {
    run()
}
