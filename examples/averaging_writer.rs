use std::env;
use std::io;
use std::path;
use std::thread;
use std::time::Instant;

use std::sync::mpsc::channel;

use rayon::prelude::*;

use mzdata::prelude::*;
use mzdata::spectrum::utils::Collator;
use mzdata::{MzMLReader, MzMLWriter};

fn main() -> io::Result<()> {
    let path = path::PathBuf::from(
        env::args()
            .nth(1)
            .expect("Please pass an MS data file path"),
    );

    let mut reader = MzMLReader::open_path(path)?;
    let mut writer = MzMLWriter::new(io::BufWriter::new(io::stdout()));
    writer.copy_metadata_from(&reader);

    let (input_sender, input_receiver) = channel();
    let (output_sender, output_receiver) = channel();

    let start = Instant::now();
    let reader_task = thread::spawn(move || {
        let (grouper, averager, _reprofiler) =
            reader.groups().averaging_deferred(1, 120.0, 2000.1, 0.005);
        grouper
            .enumerate()
            .par_bridge()
            .map_init(
                || averager.clone(),
                |averager, (i, g)| {
                    let (mut g, arrays) = g.average_with(averager);
                    g.precursor_mut().and_then(|p| {
                        p.arrays = Some(arrays.into());
                        Some(())
                    });
                    (i, g)
                },
            )
            .for_each(|(i, g)| {
                input_sender.send((i, g)).unwrap();
            });
        let end_read = Instant::now();
        eprintln!(
            "Finished reading all spectra and averaging in {:0.3?}",
            end_read - start
        );
    });

    let collator_task = thread::spawn(move || Collator::collate(input_receiver, output_sender));

    let writer_task = thread::spawn(move || -> io::Result<()> {
        for (_, group) in output_receiver {
            writer.write_group(&group)?;
        }
        writer.close().unwrap();
        let end_write = Instant::now();
        eprintln!("Finished writing all spectra in {:0.3?}", end_write - start);
        Ok(())
    });

    if let Err(e) = reader_task.join() {
        eprintln!("An error occurred while joining processing spectra task: {e:?}")
    }

    if let Err(e) = collator_task.join() {
        eprintln!("An error occurred while joining collating spectra task: {e:?}")
    }

    match writer_task.join() {
        Ok(r) => {
            r?;
        }
        Err(e) => {
            eprintln!("An error occurred while joining writing spectra task: {e:?}")
        }
    }

    Ok(())
}
