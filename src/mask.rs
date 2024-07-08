use super::PLUGIN_NAME;
use failure::Error;
use std::ptr;
use vapoursynth::core::CoreRef;
use vapoursynth::format::ColorFamily;
use vapoursynth::plugins::{Filter, FrameContext};
use vapoursynth::prelude::*;
use vapoursynth::video_info::{Property, VideoInfo};

pub struct Mask<'core> {
    pub source: Node<'core>,
    pub luma_scaling: f32,
}

#[rustfmt::skip]
static FLOAT_RANGE: [f32; 256] = [0.0, 0.003921569, 0.007843138, 0.011764706, 0.015686275, 0.019607844, 0.023529412, 0.02745098, 0.03137255, 0.03529412, 0.039215688, 0.043137256, 0.047058824, 0.050980393, 0.05490196, 0.05882353, 0.0627451, 0.06666667, 0.07058824, 0.07450981, 0.078431375, 0.08235294, 0.08627451, 0.09019608, 0.09411765, 0.09803922, 0.101960786, 0.105882354, 0.10980392, 0.11372549, 0.11764706, 0.12156863, 0.1254902, 0.12941177, 0.13333334, 0.13725491, 0.14117648, 0.14509805, 0.14901961, 0.15294118, 0.15686275, 0.16078432, 0.16470589, 0.16862746, 0.17254902, 0.1764706, 0.18039216, 0.18431373, 0.1882353, 0.19215687, 0.19607843, 0.2, 0.20392157, 0.20784314, 0.21176471, 0.21568628, 0.21960784, 0.22352941, 0.22745098, 0.23137255, 0.23529412, 0.23921569, 0.24313726, 0.24705882, 0.2509804, 0.25490198, 0.25882354, 0.2627451, 0.26666668, 0.27058825, 0.27450982, 0.2784314, 0.28235295, 0.28627452, 0.2901961, 0.29411766, 0.29803923, 0.3019608, 0.30588236, 0.30980393, 0.3137255, 0.31764707, 0.32156864, 0.3254902, 0.32941177, 0.33333334, 0.3372549, 0.34117648, 0.34509805, 0.34901962, 0.3529412, 0.35686275, 0.36078432, 0.3647059, 0.36862746, 0.37254903, 0.3764706, 0.38039216, 0.38431373, 0.3882353, 0.39215687, 0.39607844, 0.4, 0.40392157, 0.40784314, 0.4117647, 0.41568628, 0.41960785, 0.42352942, 0.42745098, 0.43137255, 0.43529412, 0.4392157, 0.44313726, 0.44705883, 0.4509804, 0.45490196, 0.45882353, 0.4627451, 0.46666667, 0.47058824, 0.4745098, 0.47843137, 0.48235294, 0.4862745, 0.49019608, 0.49411765, 0.49803922, 0.5019608, 0.5058824, 0.50980395, 0.5137255, 0.5176471, 0.52156866, 0.5254902, 0.5294118, 0.53333336, 0.5372549, 0.5411765, 0.54509807, 0.54901963, 0.5529412, 0.5568628, 0.56078434, 0.5647059, 0.5686275, 0.57254905, 0.5764706, 0.5803922, 0.58431375, 0.5882353, 0.5921569, 0.59607846, 0.6, 0.6039216, 0.60784316, 0.6117647, 0.6156863, 0.61960787, 0.62352943, 0.627451, 0.6313726, 0.63529414, 0.6392157, 0.6431373, 0.64705884, 0.6509804, 0.654902, 0.65882355, 0.6627451, 0.6666667, 0.67058825, 0.6745098, 0.6784314, 0.68235296, 0.6862745, 0.6901961, 0.69411767, 0.69803923, 0.7019608, 0.7058824, 0.70980394, 0.7137255, 0.7176471, 0.72156864, 0.7254902, 0.7294118, 0.73333335, 0.7372549, 0.7411765, 0.74509805, 0.7490196, 0.7529412, 0.75686276, 0.7607843, 0.7647059, 0.76862746, 0.77254903, 0.7764706, 0.78039217, 0.78431374, 0.7882353, 0.7921569, 0.79607844, 0.8, 0.8039216, 0.80784315, 0.8117647, 0.8156863, 0.81960785, 0.8235294, 0.827451, 0.83137256, 0.8352941, 0.8392157, 0.84313726, 0.84705883, 0.8509804, 0.85490197, 0.85882354, 0.8627451, 0.8666667, 0.87058824, 0.8745098, 0.8784314, 0.88235295, 0.8862745, 0.8901961, 0.89411765, 0.8980392, 0.9019608, 0.90588236, 0.9098039, 0.9137255, 0.91764706, 0.92156863, 0.9254902, 0.92941177, 0.93333334, 0.9372549, 0.9411765, 0.94509804, 0.9490196, 0.9529412, 0.95686275, 0.9607843, 0.9647059, 0.96862745, 0.972549, 0.9764706, 0.98039216, 0.9843137, 0.9882353, 0.99215686, 0.99607843, 1.0];

#[inline]
pub fn get_mask_value(x: f32, luma_scaling: f32) -> f32 {
    f32::powf(
        1.0 - (x
            * (x.mul_add(
                x.mul_add(x.mul_add(x.mul_add(18.188, -45.47), 36.624), -9.466),
                1.124,
            ))),
        luma_scaling,
    )
}

#[inline]
pub fn get_mask_value_clamping(x: f32, luma_scaling: f32) -> f32 {
    get_mask_value(x.min(1.0).max(0.0), luma_scaling)
}

macro_rules! from_property {
    ($prop: expr) => {
        match $prop {
            Property::Constant(p) => p,
            Property::Variable => unreachable!(),
        }
    };
}

macro_rules! int_filter {
    ($type:ty, $fname:ident) => {
        fn $fname(frame: &mut FrameRefMut, src_frame: FrameRef, depth: u8, luma_scaling: f32) {
            let max = ((1 << depth) - 1) as f32;
            let lut: Vec<$type> = FLOAT_RANGE
                .iter()
                .map(|x| (get_mask_value(*x, luma_scaling) * max) as $type)
                .collect();
            for row in 0..frame.height(0) {
                for (pixel, src_pixel) in frame
                    .plane_row_mut::<$type>(0, row)
                    .iter_mut()
                    .zip(src_frame.plane_row::<$type>(0, row))
                {
                    let i = (src_pixel >> (depth - 8)) as usize;
                    unsafe {
                        ptr::write(pixel, lut[i].clone());
                    }
                }
            }
        }
    };
}

fn filter_for_float(frame: &mut FrameRefMut, src_frame: FrameRef, luma_scaling: f32) {
    for row in 0..frame.height(0) {
        frame
            .plane_row_mut::<f32>(0, row)
            .iter_mut()
            .zip(src_frame.plane_row::<f32>(0, row))
            .for_each(|(pixel, src_pixel)| unsafe {
                ptr::write(pixel, get_mask_value(*src_pixel, luma_scaling));
            });
    }
}

fn filter_for_float_clamping(frame: &mut FrameRefMut, src_frame: FrameRef, luma_scaling: f32) {
    for row in 0..frame.height(0) {
        frame
            .plane_row_mut::<f32>(0, row)
            .iter_mut()
            .zip(src_frame.plane_row::<f32>(0, row))
            .for_each(|(pixel, src_pixel)| unsafe {
                ptr::write(pixel, get_mask_value_clamping(*src_pixel, luma_scaling));
            });
    }
}

impl<'core> Filter<'core> for Mask<'core> {
    fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        let info = self.source.info();
        let format = match info.format {
            Property::Variable => unreachable!(),
            Property::Constant(format) => format,
        };
        vec![VideoInfo {
            format: Property::Constant(
                _core
                    .register_format(
                        ColorFamily::Gray,
                        format.sample_type(),
                        format.bits_per_sample(),
                        0,
                        0,
                    )
                    .unwrap(),
            ),
            flags: info.flags,
            framerate: info.framerate,
            num_frames: info.num_frames,
            resolution: info.resolution,
        }]
    }

    fn get_frame_initial(
        &self,
        _api: API,
        _core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<Option<FrameRef<'core>>, Error> {
        self.source.request_frame_filter(context, n);
        Ok(None)
    }

    fn get_frame(
        &self,
        _api: API,
        core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>, Error> {
        let new_format = from_property!(self.video_info(_api, core)[0].format);
        let mut frame = unsafe {
            FrameRefMut::new_uninitialized(
                core,
                None,
                new_format,
                from_property!(self.source.info().resolution),
            )
        };
        let src_frame = self.source.get_frame_filter(context, n).ok_or_else(|| {
            format_err!("Could not retrieve source frame. This shouldn’t happen.")
        })?;
        let props = src_frame.props();
        let average = match props.get::<f64>("PlaneStatsAverage") {
            Ok(average) => average as f32,
            Err(_) => bail!(format!(
                "{}: you need to run std.PlaneStats on the clip before calling this function.",
                PLUGIN_NAME
            )),
        };

        match from_property!(self.source.info().format).sample_type() {
            SampleType::Integer => {
                let depth = from_property!(self.source.info().format).bits_per_sample();
                match depth {
                    0..=8 => {
                        int_filter!(u8, filter_8bit);
                        filter_8bit(
                            &mut frame,
                            src_frame,
                            depth,
                            calc_luma_scaling(average, self.luma_scaling),
                        )
                    }
                    9..=16 => {
                        int_filter!(u16, filter_16bit);
                        filter_16bit(
                            &mut frame,
                            src_frame,
                            depth,
                            calc_luma_scaling(average, self.luma_scaling),
                        )
                    }
                    17..=32 => {
                        int_filter!(u32, filter_32bit);
                        filter_32bit(
                            &mut frame,
                            src_frame,
                            depth,
                            calc_luma_scaling(average, self.luma_scaling),
                        )
                    }
                    _ => bail!(format!(
                        "{}: input depth {} not supported",
                        PLUGIN_NAME, depth
                    )),
                }
            }
            SampleType::Float => {
                // If the input has pixel values outside of the valid range (0-1),
                // those might also be out of range in the output.
                // We use the min/max props to determine if output clamping is necessary.
                let max = props
                    .get::<f64>("PlaneStatsMax")
                    .expect(&format!("{}: no PlaneStatsMax in frame props", PLUGIN_NAME));
                let min = props
                    .get::<f64>("PlaneStatsMin")
                    .expect(&format!("{}: no PlaneStatsMin in frame props", PLUGIN_NAME));
                if max > 1.0 || min < 0.0 {
                    filter_for_float_clamping(
                        &mut frame,
                        src_frame,
                        calc_luma_scaling(average, self.luma_scaling),
                    );
                } else {
                    filter_for_float(
                        &mut frame,
                        src_frame,
                        calc_luma_scaling(average, self.luma_scaling),
                    );
                }
            }
        }
        Ok(frame.into())
    }
}

pub fn calc_luma_scaling(average: f32, luma_scaling: f32) -> f32 {
    let average = average.min(1.0).max(0.0);
    average * average * luma_scaling
}

#[cfg(test)]
mod tests {
    use super::*;

    // Just in case this isn’t the last time I rewrite the lut builder:
    #[rustfmt::skip]
    static EXPECTED_MASK_02: [f32; 256] = [1.0, 0.998292, 0.99669147, 0.99519384, 0.9937948, 0.99249005, 0.99127513, 0.99014574, 0.98909765, 0.98812664, 0.9872284, 0.9863988, 0.98563385, 0.9849295, 0.9842816, 0.98368645, 0.9831401, 0.9826388, 0.9821788, 0.9817565, 0.98136836, 0.98101085, 0.98068064, 0.98037434, 0.9800887, 0.97982067, 0.979567, 0.97932476, 0.9790909, 0.9788629, 0.97863764, 0.97841257, 0.97818506, 0.9779526, 0.9777127, 0.977463, 0.97720116, 0.97692496, 0.97663224, 0.97632086, 0.97598886, 0.9756343, 0.9752552, 0.97484976, 0.97441626, 0.973953, 0.9734583, 0.97293067, 0.9723685, 0.97177035, 0.9711349, 0.9704607, 0.96974653, 0.96899116, 0.9681933, 0.9673519, 0.9664658, 0.965534, 0.9645555, 0.9635293, 0.96245456, 0.9613303, 0.96015584, 0.9589302, 0.9576528, 0.9563228, 0.95493966, 0.9535026, 0.9520111, 0.95046455, 0.9488624, 0.9472042, 0.94548935, 0.94371754, 0.9418882, 0.94000113, 0.9380558, 0.936052, 0.93398947, 0.9318677, 0.9296866, 0.9274459, 0.9251454, 0.92278486, 0.92036426, 0.91788334, 0.91534203, 0.91274023, 0.9100778, 0.9073548, 0.9045712, 0.9017268, 0.89882183, 0.8958562, 0.89282995, 0.8897432, 0.886596, 0.8833886, 0.88012075, 0.876793, 0.8734053, 0.8699578, 0.8664507, 0.8628842, 0.85925865, 0.8555742, 0.85183114, 0.8480296, 0.84417003, 0.8402527, 0.836278, 0.8322462, 0.82815754, 0.8240127, 0.81981164, 0.8155552, 0.8112436, 0.8068773, 0.8024568, 0.79798263, 0.7934552, 0.788875, 0.78424263, 0.77955866, 0.77482367, 0.7700382, 0.76520306, 0.7603186, 0.7553858, 0.750405, 0.74537706, 0.7403031, 0.73518324, 0.73001873, 0.72480994, 0.7195582, 0.714264, 0.7089286, 0.70355237, 0.6981368, 0.6926824, 0.6871906, 0.68166244, 0.6760984, 0.67050064, 0.66486925, 0.65920633, 0.6535124, 0.64778924, 0.6420377, 0.63625985, 0.6304561, 0.62462854, 0.6187791, 0.61290854, 0.60701925, 0.60111195, 0.5951895, 0.58925253, 0.5833042, 0.57734525, 0.5713786, 0.5654054, 0.55942863, 0.5534506, 0.5474728, 0.54149866, 0.5355294, 0.5295689, 0.52361876, 0.51768285, 0.5117627, 0.50586176, 0.49998415, 0.49413142, 0.48830834, 0.48251665, 0.47676167, 0.4710447, 0.4653721, 0.45974478, 0.45416957, 0.44864753, 0.44318435, 0.43778518, 0.4324514, 0.4271907, 0.42200384, 0.4168986, 0.41187632, 0.40694392, 0.4021045, 0.39736322, 0.39272374, 0.38819104, 0.38376838, 0.37945992, 0.3752701, 0.37120032, 0.36725762, 0.36344275, 0.35975853, 0.3562081, 0.35279226, 0.34951332, 0.3463715, 0.3433669, 0.34050032, 0.33777085, 0.3351737, 0.3327109, 0.33037695, 0.32816976, 0.32608336, 0.32411364, 0.32225302, 0.32049632, 0.31883442, 0.31725863, 0.31575835, 0.31432915, 0.31295443, 0.31162542, 0.31032994, 0.30905154, 0.30778122, 0.30649903, 0.30519295, 0.3038461, 0.30243647, 0.3009518, 0.29937094, 0.29767165, 0.29583326, 0.2938297, 0.29163584, 0.28922495, 0.28656486, 0.28362364, 0.28036165, 0.27673277, 0.2726987, 0.26819843, 0.2631695, 0.25753176, 0.25119466, 0.24403761, 0.23592123, 0.2266435, 0.2159501, 0.20344463, 0.18856688, 0.17031908, 0.1467575, 0.11270576, 0.00323677];
    #[rustfmt::skip]
    static EXPECTED_MASK_08: [f32; 256] = [1.0, 0.97301966, 0.9483565, 0.92581207, 0.9052065, 0.8863773, 0.86917543, 0.8534658, 0.8391256, 0.82604086, 0.81410825, 0.80323166, 0.79332286, 0.78429985, 0.7760863, 0.7686117, 0.7618097, 0.7556182, 0.7499784, 0.7448358, 0.74013805, 0.735836, 0.7318828, 0.7282341, 0.724847, 0.7216812, 0.7186976, 0.7158591, 0.7131299, 0.71047634, 0.70786506, 0.705265, 0.7026458, 0.6999789, 0.69723636, 0.69439274, 0.6914226, 0.68830246, 0.6850099, 0.6815241, 0.6778255, 0.6738958, 0.66971844, 0.6652776, 0.6605602, 0.65555304, 0.650246, 0.6446294, 0.6386957, 0.6324387, 0.6258541, 0.6189384, 0.6116908, 0.60411143, 0.59620154, 0.5879653, 0.5794071, 0.5705335, 0.5613522, 0.55187273, 0.5421053, 0.532062, 0.52175605, 0.5112014, 0.5004138, 0.4894094, 0.47820586, 0.46682078, 0.45527333, 0.44358265, 0.43176922, 0.41985318, 0.4078552, 0.39579678, 0.38369834, 0.37158158, 0.35946706, 0.3473761, 0.33532932, 0.3233465, 0.31144762, 0.29965213, 0.28797832, 0.27644426, 0.2650672, 0.2538632, 0.24284792, 0.23203576, 0.22144021, 0.21107364, 0.20094803, 0.19107313, 0.18145862, 0.17211263, 0.1630422, 0.15425344, 0.14575136, 0.13753979, 0.12962131, 0.121998124, 0.11467082, 0.107639365, 0.100902446, 0.09445834, 0.08830449, 0.08243708, 0.076852135, 0.07154449, 0.06650879, 0.061738882, 0.05722833, 0.052969858, 0.048956152, 0.045179587, 0.041631833, 0.03830488, 0.035190117, 0.032279003, 0.029562855, 0.027033038, 0.02468074, 0.022497335, 0.020474205, 0.018602902, 0.016875094, 0.015282571, 0.013817431, 0.012471816, 0.0112383105, 0.0101095475, 0.009078545, 0.008138663, 0.0072833193, 0.006506449, 0.0058021126, 0.0051648114, 0.004589231, 0.004070427, 0.0036036533, 0.0031845558, 0.0028089648, 0.0024730647, 0.0021732512, 0.0019061574, 0.0016687337, 0.0014580758, 0.0012715748, 0.001106781, 0.0009614784, 0.0008336202, 0.00072136015, 0.00062298466, 0.00053696934, 0.00046192406, 0.00039657977, 0.00033981237, 0.00029059322, 0.00024801854, 0.00021126306, 0.00017960659, 0.00015239452, 0.00012905747, 0.0001090837, 0.000092026974, 0.000077492776, 0.00006513292, 0.000054645934, 0.00004576495, 0.00003826129, 0.000031933254, 0.000026608495, 0.000022135933, 0.000018386958, 0.000015251044, 0.000012632214, 0.000010449756, 0.000008633694, 0.000007125597, 0.000005874811, 0.0000048395545, 0.000003983536, 0.0000032770909, 0.0000026945397, 0.0000022148818, 0.0000018204425, 0.0000014962245, 0.000001230123, 0.000001011725, 0.00000083269754, 0.00000068591316, 0.0000005656565, 0.00000046711415, 0.0000003863694, 0.00000032018005, 0.00000026590607, 0.00000022136388, 0.00000018477976, 0.0000001547046, 0.00000012993713, 0.000000109530006, 0.000000092677915, 0.00000007873675, 0.00000006718318, 0.00000005758441, 0.00000004959293, 0.000000042921464, 0.000000037336587, 0.000000032649805, 0.000000028704813, 0.0000000253699, 0.000000022546159, 0.00000002014448, 0.000000018095754, 0.000000016340236, 0.000000014830539, 0.000000013525465, 0.000000012392772, 0.00000001140364, 0.000000010534523, 0.0000000097651105, 0.000000009081434, 0.000000008466379, 0.000000007909084, 0.000000007399097, 0.000000006926191, 0.000000006484458, 0.0000000060654903, 0.000000005664892, 0.000000005277865, 0.00000000489944, 0.0000000045284674, 0.0000000041625015, 0.0000000038001398, 0.0000000034415308, 0.0000000030869618, 0.000000002738131, 0.000000002397576, 0.0000000020680777, 0.0000000017533902, 0.0000000014571282, 0.0000000011829588, 0.0000000009352538, 0.00000000071663736, 0.0000000005293817, 0.00000000037436287, 0.00000000025128466, 0.00000000015823709, 0.000000000092102735, 0.000000000048472164, 0.000000000022369412, 0.000000000008612706, 0.000000000002555328, 0.00000000000050143293, 0.00000000000004630252, 0.0000000000000006778562, 0.000000000000000000000000000000000000000145141];

    #[test]
    fn test_mask_values() {
        FLOAT_RANGE
            .iter()
            .zip(EXPECTED_MASK_02.iter())
            .for_each(|(&x, &exp)| {
                let value = get_mask_value(x, calc_luma_scaling(0.2, 10.0));
                assert!(
                    (value - exp).abs() < 0.0001,
                    "luma scaling 0.2: Mask was wrong at position {}, expected {}, got {}",
                    x,
                    exp,
                    value
                );
            });
        FLOAT_RANGE
            .iter()
            .zip(EXPECTED_MASK_08.iter())
            .for_each(|(&x, &exp)| {
                let value = get_mask_value(x, calc_luma_scaling(0.8, 10.0));
                assert!(
                    (value - exp).abs() < 0.0001,
                    "luma scaling 0.8: Mask was wrong at position {}, expected {}, got {}",
                    x,
                    exp,
                    value
                );
            });
    }

    #[test]
    fn test_mask_values_clamping() {
        FLOAT_RANGE
            .iter()
            .zip(EXPECTED_MASK_02.iter())
            .for_each(|(&x, &exp)| {
                assert!(
                    (get_mask_value_clamping(x, calc_luma_scaling(0.2, 10.0)) - exp).abs() < 0.0001
                );
            });
        assert_eq!(
            get_mask_value_clamping(1.1, calc_luma_scaling(0.99, 10.0)),
            0.0
        );
        assert_eq!(
            get_mask_value_clamping(-0.1, calc_luma_scaling(-0.1, 10.0)),
            1.0
        );
    }
}