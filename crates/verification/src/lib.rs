mod utils;

use std::{fs, io::Read, path::Path};

use anyhow::Result;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};

pub(crate) const SIGNATURE: &str = "verification";
pub(crate) const EXCLUDES: &[&str] = &["meta-mm", "customize.sh"];

pub fn generate_key() -> Result<(SigningKey, VerifyingKey)> {
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed)?;
    let priv_key = SigningKey::from_bytes(&seed);
    let pub_key = priv_key.verifying_key();
    Ok((priv_key, pub_key))
}

pub fn sign<P>(sign_key: &SigningKey, path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let files = utils::read_files(path.as_ref())?;
    let mut all_hash = Vec::new();

    for file in files {
        all_hash.extend(utils::get_hash(file)?);
    }

    let signature = sign_key.sign(&all_hash);
    fs::write(path.as_ref().join(SIGNATURE), signature.to_bytes())?;

    Ok(())
}

pub fn verify<P>(verif_key: &VerifyingKey, path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let sig_path = path.as_ref().join(SIGNATURE);
    let mut file = fs::OpenOptions::new().read(true).open(&sig_path)?;
    let mut buf = [0u8; ed25519_dalek::SIGNATURE_LENGTH];
    file.read_exact(&mut buf)?;
    let files = utils::read_files(path.as_ref())?;
    let mut all_hash = Vec::new();

    for file in files {
        all_hash.extend(utils::get_hash(file)?);
    }

    let signature = Signature::from_bytes(&buf);
    verif_key.verify_strict(&all_hash, &signature)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use anyhow::Result;

    use crate::SIGNATURE;

    use super::{generate_key, sign, verify};

    #[test]
    fn test_generate_key() -> Result<()> {
        let (sk, vk) = generate_key()?;
        assert_eq!(sk.to_bytes().len(), 32);
        assert_eq!(vk.to_bytes().len(), 32);
        Ok(())
    }

    #[test]
    fn test_sign_and_verify() -> Result<()> {
        let dir = tempfile::Builder::new().tempdir()?;
        let dir_path = dir.path();

        let mut f1 = fs::File::create(dir_path.join("test"))?;
        f1.write_all(b"test")?;
        let mut f2 = fs::File::create(dir_path.join("test1"))?;
        f2.write_all(b"test1")?;

        let (sk, vk) = generate_key()?;
        assert!(sign(&sk, dir_path).is_ok());
        assert!(dir_path.join(SIGNATURE).exists());

        assert!(verify(&vk, dir_path).is_ok());
        Ok(())
    }

    #[test]
    fn test_sign_and_verify_with_excludes() -> Result<()> {
        let dir = tempfile::Builder::new().tempdir()?;
        let dir_path = dir.path();

        let mut f1 = std::fs::File::create(dir_path.join("test"))?;
        f1.write_all(b"test")?;
        let mut f2 = std::fs::File::create(dir_path.join("test1"))?;
        f2.write_all(b"test1")?;

        fs::File::create(dir_path.join("customize.sh"))?;

        let (sk, vk) = generate_key()?;
        assert!(sign(&sk, dir_path).is_ok());
        assert!(dir_path.join(SIGNATURE).exists());

        std::fs::remove_file(dir_path.join("customize.sh"))?;

        assert!(verify(&vk, dir_path).is_ok());
        Ok(())
    }

    #[test]
    fn test_verify_missing_signature() {
        let dir = tempfile::Builder::new().tempdir().unwrap();
        let dir_path = dir.path();

        let mut f1 = std::fs::File::create(dir_path.join("test")).unwrap();
        f1.write_all(b"test").unwrap();

        let (_, vk) = generate_key().unwrap();
        assert!(verify(&vk, dir_path).is_err());
    }
}
