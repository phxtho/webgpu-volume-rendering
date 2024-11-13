// original source: https://github.com/davazp/webgpu-and-volume-rendering/blob/main/src/api/dicom.ts

pub type Vec3 = [f32; 3];

#[derive(Debug)]
#[allow(dead_code)]
pub struct ImageVolume {
    pub columns: u16,
    pub rows: u16,
    pub slices: usize,
    pub pixel_spacing: Vec3,
    pub position_patient: Vec3,
    pub image_orientation_patient: [Vec3; 3],
    pub volume: Vec<f32>,
}

use anyhow::{anyhow, Result};
use dicom::object::{open_file, DefaultDicomObject};
use std::path::Path;

struct DicomSlice {
    columns: u16,
    rows: u16,
    slice_location: f32,
    pixel_spacing: [f32; 2],
    position_patient: Vec3,
    image_orientation_patient: [Vec3; 2],
    image: Vec<f32>,
}

pub fn load_dicom_image<P: AsRef<Path>>(files: &[P]) -> Result<ImageVolume> {
    let mut slices: Vec<DicomSlice> = files
        .iter()
        .map(|file| read_single_image(file))
        .collect::<Result<Vec<_>>>()?;

    // Sort slices by location
    slices.sort_by(|a, b| a.slice_location.partial_cmp(&b.slice_location).unwrap());

    if slices.len() < 2 {
        return Err(anyhow!("Need at least two slices"));
    }

    let first_slice = slices.first().unwrap();
    let last_slice = slices.last().unwrap();

    // Verify consistent dimensions
    let columns = first_slice.columns;
    let rows = first_slice.rows;
    let pixel_spacing_2d = first_slice.pixel_spacing;

    // Calculate interslice spacing
    let interslice_vector = [
        (last_slice.position_patient[0] - first_slice.position_patient[0])
            / (slices.len() - 1) as f32,
        (last_slice.position_patient[1] - first_slice.position_patient[1])
            / (slices.len() - 1) as f32,
        (last_slice.position_patient[2] - first_slice.position_patient[2])
            / (slices.len() - 1) as f32,
    ];

    let pixel_spacing_z = (interslice_vector[0].powi(2)
        + interslice_vector[1].powi(2)
        + interslice_vector[2].powi(2))
    .sqrt();

    let pixel_spacing = [pixel_spacing_2d[0], pixel_spacing_2d[1], pixel_spacing_z];

    // Combine all slice data into volume
    let slice_size = (columns as usize) * (rows as usize);
    let mut volume = Vec::with_capacity(slice_size * slices.len());

    for slice in &slices {
        volume.extend(&slice.image);
    }

    let image_orientation_patient = [
        first_slice.image_orientation_patient[0],
        first_slice.image_orientation_patient[1],
        [
            interslice_vector[0] / pixel_spacing_z,
            interslice_vector[1] / pixel_spacing_z,
            interslice_vector[2] / pixel_spacing_z,
        ],
    ];

    Ok(ImageVolume {
        columns,
        rows,
        slices: slices.len(),
        pixel_spacing,
        position_patient: first_slice.position_patient,
        image_orientation_patient,
        volume,
    })
}

fn read_single_image<P: AsRef<Path>>(file: P) -> Result<DicomSlice> {
    let obj: DefaultDicomObject = open_file(file)?;

    let columns = obj.element_by_name("Columns")?.uint16()?;
    let rows = obj.element_by_name("Rows")?.uint16()?;
    // Read position patient
    let position_patient: [f32; 3] = obj
        .element_by_name("ImagePositionPatient")?
        .to_multi_float32()?
        .try_into()
        .map_err(|_| anyhow!("Invalid ImagePositionPatient length"))?;

    let slice_location = obj
        .element_by_name("SliceLocation")
        .map_or(position_patient[2], |elem| elem.to_float32().unwrap());

    // Read pixel spacing
    let pixel_spacing: [f32; 2] = obj
        .element_by_name("PixelSpacing")?
        .to_multi_float32()?
        .try_into()
        .map_err(|_| anyhow!("Invalid PixelSpacing length"))?;

    // Read orientation
    let orientation: Vec<f32> = obj
        .element_by_name("ImageOrientationPatient")?
        .to_multi_float32()?;

    let image_orientation_patient = [
        [orientation[0], orientation[1], orientation[2]],
        [orientation[3], orientation[4], orientation[5]],
    ];

    // Read pixel data and apply rescale
    let pixel_data = obj.element_by_name("PixelData")?;
    let rescale_intercept = obj
        .element_by_name("RescaleIntercept")
        .map_or(0.0, |elem| elem.to_float32().unwrap_or(0.0));
    let rescale_slope = obj
        .element_by_name("RescaleSlope")
        .map_or(1.0, |elem| elem.to_float32().unwrap_or(1.0));

    // Convert pixel data to f32 with rescale
    let image: Vec<f32> = pixel_data
        .to_multi_int::<i16>()?
        .iter()
        .map(|&x| (x as f32) * rescale_slope + rescale_intercept)
        .collect();

    Ok(DicomSlice {
        columns,
        rows,
        slice_location,
        pixel_spacing,
        position_patient,
        image_orientation_patient,
        image,
    })
}
