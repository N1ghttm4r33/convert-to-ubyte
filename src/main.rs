use image::{GenericImageView, Luma};
use image::imageops;
use tokio::task;
use ndarray::Array2;
use ndarray::prelude::*;
use ndarray_npy::write_npy;
use std::fs::File;

async fn process_image(image_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut img = image::open(image_path)?.to_luma8();

    // Corta 300px e deixa apenas os primeiros 420px
    let img = img.view(0, 300, img.width(), 420).to_image();

    let img_threshold = img.map(|_, _, p| if p[0] > 150 { Luma([255u8]) } else { Luma([0u8]) });

    let img_blur = imageops::blur(&img_threshold, 5.0);

    // usa ocrs para extrair o texto
    let mut ocr = ocrs::new(None, Some("eng"))?;
    ocr.set_image_from_memory(&img_blur)?;
    let texto = ocr.get_text()?;

    let linhas: Vec<&str> = texto.split('\n').collect();

    let texto: Vec<String> = linhas.iter().map(|linha| linha.trim().to_string()).collect();

    let texto_concatenado = texto.join(" ");

    let img = image::open(image_path)?.to_luma8();
    let img = img.view(0, 720, img.width(), 2032 - 720).to_image();

    let img = image::imageops::resize(&img, 2447, 1280, image::imageops::FilterType::Nearest);

    // Salva a imagem como um arquivo ubyte
    let img_array: Array2<u8> = Array::from_shape_vec((img.height() as usize, img.width() as usize), img.into_raw()).unwrap();
    let writer = File::create("image.ubyte")?;
    write_npy(writer, &img_array)?;

    Ok(())
}

#[tokio::main]
async fn main() {
    match task::spawn(process_image("path/to/your/image.png")).await {
        Ok(Ok(_)) => println!("Processamento concluído com sucesso."),
        Ok(Err(e)) => println!("Erro ao processar a imagem: {}", e),
        Err(e) => println!("Erro ao iniciar a tarefa: {}", e),
    }
}
