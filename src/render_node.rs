use crate::{
    shader::{FragmentShader, VertexShader},
    texture::Texture,
    uniforms::UniformBindGroup,
};
use crate::{RenderError, VertexBufferData, GpuData};
use smallvec::SmallVec;
use std::any::TypeId;

const VERTX_BUFFER_STACK_LIMIT: usize = 3;

pub struct RenderNode<'a> {
    vertex_buffers:
        SmallVec<[(TypeId, wgpu::VertexBufferDescriptor<'a>); VERTX_BUFFER_STACK_LIMIT]>,
    uniform_bind_groups: Vec<UniformBindGroup>,
    vertex_shader: VertexShader,
    fragment_shader: Option<FragmentShader>,
    pipeline: wgpu::RenderPipeline,
    //    func: T
    //    textures: Vec<dyn Texture>
}

#[derive(Default)]
pub struct RenderNodeBuilder<'a> {
    vertex_buffers:
        SmallVec<[(TypeId, wgpu::VertexBufferDescriptor<'a>); VERTX_BUFFER_STACK_LIMIT]>,
    uniform_bind_groups: Vec<UniformBindGroup>,
    vertex_shader: Option<VertexShader>,
    fragment_shader: Option<FragmentShader>,
    //  func: Option<T>,
}

impl<'a> RenderNodeBuilder<'a> {
    pub fn add_vertex_buffer<VB: VertexBufferData>(mut self) -> Self {
        self.vertex_buffers
            .push((TypeId::of::<VB>(), VB::get_descriptor()));
        self
    }

    pub fn add_uniform_bind_group(mut self, uniform: UniformBindGroup) -> Self {
        self.uniform_bind_groups.push(uniform);
        self
    }

    pub fn set_vertex_shader(mut self, vertex_shader: VertexShader) -> Self {
        self.vertex_shader = Some(vertex_shader);
        self
    }

    pub fn set_fragment_shader(mut self, fragment_shader: FragmentShader) -> Self {
        self.fragment_shader = Some(fragment_shader);
        self
    }

    /*    pub fn set_render_func(mut self, func: T) -> Self {
        self.func = Some(func);
        self
    }*/

    fn construct_pipeline(
        &self,
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let bind_group_layouts = self
            .uniform_bind_groups
            .iter()
            .map(UniformBindGroup::get_layout)
            .collect::<Vec<&wgpu::BindGroupLayout>>();

        // TODO: Add texture bindgroup layouts here as well

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &bind_group_layouts,
            });

        let vertex_buffer_desc = &self
            .vertex_buffers
            .iter()
            .map(|(_, descriptor)| descriptor.clone())
            .collect::<Vec<wgpu::VertexBufferDescriptor>>();

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: self.vertex_shader.as_ref().unwrap().get_descriptor(),
            fragment_stage: self
                .fragment_shader
                .as_ref()
                .map(FragmentShader::get_descriptor),
            // TODO: add customizable rasterization stage
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: color_format,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &vertex_buffer_desc,
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }

    pub fn build(
        self,
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Result<RenderNode<'a>, RenderError> {
        if self.vertex_shader.is_none() {
            Err(RenderError::MissingVertexShader)
        } else {
            let pipeline = self.construct_pipeline(device, color_format, depth_format);
            Ok(RenderNode {
                vertex_buffers: self.vertex_buffers,
                uniform_bind_groups: self.uniform_bind_groups,
                vertex_shader: self.vertex_shader.unwrap(),
                fragment_shader: self.fragment_shader,
                pipeline,
                //                func: self.func.unwrap()
            })
        }
    }
}

impl<'a> RenderNode<'a> {
    pub fn builder() -> RenderNodeBuilder<'a> {
        RenderNodeBuilder {
            //            func: None,
            vertex_buffers: SmallVec::new(),
            vertex_shader: None,
            fragment_shader: None,
            uniform_bind_groups: Vec::new(),
        }
    }

    #[inline]
    pub fn update(
        &self,
        device: &wgpu::Device,
        command_encoder: &mut wgpu::CommandEncoder,
        index: usize,
        data: &impl GpuData
    ) {
       self.uniform_bind_groups[index].update_buffer_data(device, command_encoder, data).unwrap() 
    }

    pub fn run<'b, 'c: 'b>(
        &'c self,
        command_encoder: &'b mut wgpu::CommandEncoder,
        // this could be owned
        render_pass_descriptor: wgpu::RenderPassDescriptor<'b, '_>,
    ) -> wgpu::RenderPass<'b> {
        let mut render_pass = command_encoder.begin_render_pass(&render_pass_descriptor);
        render_pass.set_pipeline(&self.pipeline);
        self.uniform_bind_groups
            .iter()
            .enumerate()
            .for_each(|(i, group)| {
                render_pass.set_bind_group(i as u32, group.get_bind_group(), &[]);
            });
        //todo!("Decide on how to give node render commands")
        //(self.func)(&mut render_pass);
        render_pass
    }
}
