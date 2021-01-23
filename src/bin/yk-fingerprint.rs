use std::env;

use rustica_keys::yubikey::ssh::ssh_cert_fetch_pubkey;
use rustica_keys::yubikey::{fetch_attestation, fetch_subject};
use rustica_keys::yubikey::{RetiredSlotId, SlotId};

use std::convert::TryFrom;

fn help() {
    println!("Print the SSH key fingerprint for all Yubikey slots");
    println!("Usage: yk-fingerprint");
}

fn main() {
    if env::args().len() > 1 {
        return help();
    }

    println!("Normal Slots:");
    for slot in [0x9a, 0x9c, 0x9e, 0x9d, 0x9e, 0xf9].iter().map(|x| *x as u8) {
        let s = SlotId::try_from(slot).unwrap();
        match (fetch_subject(s), ssh_cert_fetch_pubkey(s))  {
            (Ok(subj), Some(cert)) => {
                let attest = fetch_attestation(s);
                println!("\t{:?}:\t[Fingerprint: {}] [Attest: {}] Subject: [{}]",
                    s,
                    cert.fingerprint().hash,
                    if attest.is_some() {"Yes" } else { "No "},
                    subj
                )
            }
            _ => println!("\t{:?}:\tNo cert found", s),
        }
    }

    println!("Retired Slots:");
    for slot in 0x82..0x96_u8 {
        let slot = SlotId::Retired(RetiredSlotId::try_from(slot).unwrap());
        match (fetch_subject(slot), ssh_cert_fetch_pubkey(slot)) {
            (Ok(subj), Some(cert)) => {
                let attest = fetch_attestation(slot);
                println!("\t{:?}:\t[Fingerprint: {}] [Attest: {}] Subject: [{}]",
                    slot,
                    cert.fingerprint().hash,
                    if attest.is_some() {"Yes" } else { "No "},
                    subj
                )
            },
            _ => println!("\t{:?}:\tNo cert found", slot),
        }
    }
}