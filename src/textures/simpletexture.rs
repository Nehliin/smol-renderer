use super::{LoadableTexture, TextureData, TextureShaderLayout};
use crate::RenderError;
use image::GenericImage;
use once_cell::sync::OnceCell;
use std::marker::PhantomData;
pub struct SimpleTexture;


impl TextureShaderLayout for SimpleTexture {
    fn get_layout(device: &wgpu::Device) -> &'static wgpu::BindGroupLayout {
        static LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();
        LAYOUT.get_or_init(move || {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry::new(
                        0,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Float,
                            multisampled: false,
                        },
                    ),
                    wgpu::BindGroupLayoutEntry::new(
                        1,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::Sampler { comparison: true },
                    ),
                ],
                label: None,
            })
        })
    }
}

impl LoadableTexture for SimpleTexture {
    fn load_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<std::path::Path>,
    ) -> Result<TextureData<Self>, RenderError> {
        let img = image::open(path)?;
        let img = img.flipv();

        let rgba = img.to_rgba(); // handle formats properly
        let (width, height) = img.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb, // handle formats properly
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let texutre_copy_view = wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        };
        let texture_data_layout = wgpu::TextureDataLayout {
            offset: 0,
            bytes_per_row: 4 * width,
            rows_per_image: 0,
        };

        queue.write_texture(texutre_copy_view, &rgba.to_vec(), texture_data_layout, size);

        let view = texture.create_default_view();
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0, // related to mipmaps
            lod_max_clamp: 100.0,  // related to mipmaps
            compare: Some(wgpu::CompareFunction::Always),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Self::get_layout(device),
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("SimpleTextureBindGroup"),
        });
        let texture_data = TextureData {
            bind_group,
            sampler,
            views: vec![view],
            texture,
            _marker: PhantomData::default(),
        };
        Ok(texture_data)
    }
}
