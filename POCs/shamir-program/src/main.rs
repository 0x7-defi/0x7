use sharks::{Share, Sharks};
use rand_chacha::rand_core::SeedableRng;

fn main() {
    let sharks = Sharks(10);
    let mut rng = rand_chacha::ChaCha8Rng::from_seed([0x90; 32]);
    let dealer = sharks.dealer_rng(&[1, 2, 3, 4], &mut rng);
    let shares: Vec<Share> = dealer.take(20).collect();

    /*for share in &shares {
        dbg!(&Vec::from(share));
    }*/

    let mut corrupt_shares: Vec<Share> = Vec::new();

    shares.iter().take(2).for_each(|some_share| {
        corrupt_shares.push(some_share.to_owned());
    });

    match sharks.recover(corrupt_shares.as_slice()) {
        Ok(recovered_secret) => {
            match recovered_secret.as_slice() {
                &[1, 2, 3, 4] => println!("KEYS MATCH"),
                _ => println!("KEYS MISMATCH"),
            }
        },
        Err(e) => {
            let to_enum: SharksError = e.into();
            println!("{:?}", to_enum);
        },
    }

    match sharks.recover(shares.as_slice()) {
        Ok(recovered_secret) => {
            match recovered_secret.as_slice() {
                &[1, 2, 3, 4] => println!("KEYS MATCH"),
                _ => println!("KEYS MISMATCH"),
            }
        },
        Err(e) => {
            let to_enum: SharksError = e.into();
            println!("{:?}", to_enum);
        },
    }

    

}

#[derive(Debug)]
enum SharksError {
    SharesMustHaveSameLength,
    NotEnoughShares,
    UnresolvedError,
}

impl From<&str> for SharksError {
    fn from(error_msg: &str) -> Self {
        match error_msg {
            "All shares must have same length" => Self::SharesMustHaveSameLength,
            "Not enough shares to recover original secret" => Self::NotEnoughShares,
            _ => Self::UnresolvedError,
        }
    }
}