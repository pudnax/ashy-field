use ash::{version::DeviceV1_0, version::EntryV1_0, version::InstanceV1_0, vk};
use eyre::*;
use std::ffi::{c_void, CStr, CString};

fn main() -> Result<()> {
    color_eyre::install()?;
    let eventloop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&eventloop)?;
    let aetna = Aetna::init(window)?;
    use winit::event::{Event, WindowEvent};
    eventloop.run(move |event, _, controlflow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *controlflow = winit::event_loop::ControlFlow::Exit;
        }
        Event::MainEventsCleared => {
            // doing the work here (later)
            aetna.window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            //render here (later)
        }
        _ => {}
    });
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}

fn init_instance(
    entry: &ash::Entry,
    layer_names: &[&str],
    extension_names: &[&CStr],
) -> Result<ash::Instance> {
    let api_version = match entry.try_enumerate_instance_version()? {
        Some(version) => version,
        None => vk::make_version(1, 0, 0),
    };
    let enginename = CString::new("UnknownGameEngine")
        .map_err(|e| eyre!("Failed to create CStr from: {}", e))?;
    let appname =
        CString::new("The Black Window").map_err(|e| eyre!("Failed to create CStr from: {}", e))?;
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&appname)
        .application_version(vk::make_version(0, 0, 1))
        .engine_name(&enginename)
        .engine_version(vk::make_version(0, 42, 0))
        .api_version(api_version);
    let layer_names_c: Vec<CString> = layer_names
        .iter()
        .map(|&ln| CString::new(ln).unwrap())
        .collect();
    let layer_name_pointers: Vec<*const i8> = layer_names_c
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
    let extension_name_pointers: Vec<*const i8> = extension_names
        .iter()
        .copied()
        .chain(vec![ash::extensions::ext::DebugUtils::name()])
        .map(|s| s.as_ptr())
        .collect();
    let mut debugcreateinfo = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback));

    let instance_create_info = vk::InstanceCreateInfo::builder()
        .push_next(&mut debugcreateinfo)
        .application_info(&app_info)
        .enabled_layer_names(&layer_name_pointers)
        .enabled_extension_names(&extension_name_pointers);
    unsafe { entry.create_instance(&instance_create_info, None) }
        .wrap_err_with(|| "Failed to create instance")
}

struct DebugDongXi {
    loader: ash::extensions::ext::DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}
impl DebugDongXi {
    fn init(entry: &ash::Entry, instance: &ash::Instance) -> Result<DebugDongXi, vk::Result> {
        let debugcreateinfo = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
            .pfn_user_callback(Some(vulkan_debug_utils_callback));

        let loader = ash::extensions::ext::DebugUtils::new(entry, instance);
        let messenger = unsafe { loader.create_debug_utils_messenger(&debugcreateinfo, None)? };

        Ok(DebugDongXi { loader, messenger })
    }
}

impl Drop for DebugDongXi {
    fn drop(&mut self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None)
        };
    }
}

struct SurfaceDongXi {
    surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
}

impl SurfaceDongXi {
    fn init(
        window: &winit::window::Window,
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> Result<SurfaceDongXi, vk::Result> {
        let surface = unsafe { ash_window::create_surface(entry, instance, window, None)? };
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
        Ok(SurfaceDongXi {
            surface,
            surface_loader,
        })
    }
    fn get_capabilities(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(physical_device, self.surface)
        }
    }
    fn get_present_modes(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::PresentModeKHR>, vk::Result> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_present_modes(physical_device, self.surface)
        }
    }
    fn get_formats(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(physical_device, self.surface)
        }
    }
    fn get_physical_device_surface_support(
        &self,
        physical_device: vk::PhysicalDevice,
        queuefamilyindex: usize,
    ) -> Result<bool, vk::Result> {
        unsafe {
            self.surface_loader.get_physical_device_surface_support(
                physical_device,
                queuefamilyindex as u32,
                self.surface,
            )
        }
    }
}

impl Drop for SurfaceDongXi {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}

fn init_physical_device_and_properties(
    instance: &ash::Instance,
) -> Result<(vk::PhysicalDevice, vk::PhysicalDeviceProperties), vk::Result> {
    let phys_devs = unsafe { instance.enumerate_physical_devices()? };
    let mut chosen = None;
    for p in phys_devs {
        let properties = unsafe { instance.get_physical_device_properties(p) };
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            chosen = Some((p, properties));
        }
    }
    chosen.ok_or(vk::Result::ERROR_UNKNOWN)
}

struct QueueFamilies {
    graphics_q_index: Option<u32>,
    transfer_q_index: Option<u32>,
}
impl QueueFamilies {
    fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surfaces: &SurfaceDongXi,
    ) -> Result<QueueFamilies, vk::Result> {
        let queuefamilyproperties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let mut found_graphics_q_index = None;
        let mut found_transfer_q_index = None;
        for (index, qfam) in queuefamilyproperties.iter().enumerate() {
            if qfam.queue_count > 0
                && qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && surfaces.get_physical_device_surface_support(physical_device, index)?
            {
                found_graphics_q_index = Some(index as u32);
            }
            if qfam.queue_count > 0
                && qfam.queue_flags.contains(vk::QueueFlags::TRANSFER)
                && (found_transfer_q_index.is_none()
                    || !qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            {
                found_transfer_q_index = Some(index as u32);
            }
        }
        Ok(QueueFamilies {
            graphics_q_index: found_graphics_q_index,
            transfer_q_index: found_transfer_q_index,
        })
    }
}

struct Queues {
    graphics_queue: vk::Queue,
    transfer_queue: vk::Queue,
}

fn init_device_and_queues(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_families: &QueueFamilies,
    layer_names: &[&str],
) -> Result<(ash::Device, Queues)> {
    let layer_names_c: Result<Vec<CString>> = layer_names
        .iter()
        .map(|&ln| CString::new(ln).map_err(|e| eyre!("Failed to create CString from: {}", e)))
        .collect();
    let layer_name_pointers: Vec<*const i8> = layer_names_c?
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let priorities = [1.0f32];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_families.graphics_q_index.unwrap())
            .queue_priorities(&priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_families.transfer_q_index.unwrap())
            .queue_priorities(&priorities)
            .build(),
    ];
    let device_extension_name_pointers: Vec<*const i8> =
        vec![ash::extensions::khr::Swapchain::name().as_ptr()];
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extension_name_pointers)
        .enabled_layer_names(&layer_name_pointers);
    let logical_device =
        unsafe { instance.create_device(physical_device, &device_create_info, None)? };
    let graphics_queue =
        unsafe { logical_device.get_device_queue(queue_families.graphics_q_index.unwrap(), 0) };
    let transfer_queue =
        unsafe { logical_device.get_device_queue(queue_families.transfer_q_index.unwrap(), 0) };
    Ok((
        logical_device,
        Queues {
            graphics_queue,
            transfer_queue,
        },
    ))
}

struct SwapchainDongXi {
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    imageviews: Vec<vk::ImageView>,
}

impl SwapchainDongXi {
    fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        surfaces: &SurfaceDongXi,
        queue_families: &QueueFamilies,
        queues: &Queues,
    ) -> Result<SwapchainDongXi, vk::Result> {
        let surface_capabilities = surfaces.get_capabilities(physical_device)?;
        let surface_present_modes = surfaces.get_present_modes(physical_device)?;
        let surface_formats = surfaces.get_formats(physical_device)?;
        let queuefamilies = [queue_families.graphics_q_index.unwrap()];
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surfaces.surface)
            .min_image_count(
                3.max(surface_capabilities.min_image_count)
                    .min(surface_capabilities.max_image_count),
            )
            .image_format(surface_formats.first().unwrap().format)
            .image_color_space(surface_formats.first().unwrap().color_space)
            .image_extent(surface_capabilities.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queuefamilies)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO);
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, logical_device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        let mut swapchain_imageviews = Vec::with_capacity(swapchain_images.len());
        for image in &swapchain_images {
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let imageview_create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::B8G8R8A8_UNORM)
                .subresource_range(*subresource_range);
            let imageview =
                unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;
            swapchain_imageviews.push(imageview);
        }
        Ok(SwapchainDongXi {
            swapchain_loader,
            swapchain,
            images: swapchain_images,
            imageviews: swapchain_imageviews,
        })
    }
    unsafe fn cleanup(&mut self, logical_device: &ash::Device) {
        for iv in &self.imageviews {
            logical_device.destroy_image_view(*iv, None);
        }
        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None)
    }
}

struct Aetna {
    window: winit::window::Window,
    entry: ash::Entry,
    instance: ash::Instance,
    debug: std::mem::ManuallyDrop<DebugDongXi>,
    surfaces: std::mem::ManuallyDrop<SurfaceDongXi>,
    physical_device: vk::PhysicalDevice,
    physical_device_properties: vk::PhysicalDeviceProperties,
    queue_families: QueueFamilies,
    queues: Queues,
    device: ash::Device,
    swapchain: SwapchainDongXi,
}

impl Aetna {
    fn init(window: winit::window::Window) -> Result<Aetna> {
        let entry = ash::Entry::new()?;
        let extension_names = ash_window::enumerate_required_extensions(&window)?;

        let layer_names = vec!["VK_LAYER_KHRONOS_validation"];
        let instance = init_instance(&entry, &layer_names, &extension_names)?;
        let debug = DebugDongXi::init(&entry, &instance)?;
        let surfaces = SurfaceDongXi::init(&window, &entry, &instance)?;

        let (physical_device, physical_device_properties) =
            init_physical_device_and_properties(&instance)?;

        let queue_families = QueueFamilies::init(&instance, physical_device, &surfaces)?;

        let (logical_device, queues) =
            init_device_and_queues(&instance, physical_device, &queue_families, &layer_names)?;
        let swapchain = SwapchainDongXi::init(
            &instance,
            physical_device,
            &logical_device,
            &surfaces,
            &queue_families,
            &queues,
        )?;
        Ok(Aetna {
            window,
            entry,
            instance,
            debug: std::mem::ManuallyDrop::new(debug),
            surfaces: std::mem::ManuallyDrop::new(surfaces),
            physical_device,
            physical_device_properties,
            queue_families,
            queues,
            device: logical_device,
            swapchain,
        })
    }
}

impl Drop for Aetna {
    fn drop(&mut self) {
        unsafe {
            self.swapchain.cleanup(&self.device);
            self.device.destroy_device(None);
            std::mem::ManuallyDrop::drop(&mut self.surfaces);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None)
        };
    }
}
