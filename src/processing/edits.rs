use imagepipe::Pipeline;

use crate::models::EditParams;

/// Apply EditParams to an imagepipe Pipeline before processing.
pub fn apply_edits(pipeline: &mut Pipeline, edits: &EditParams) {
    if let Some(ev) = edits.exposure {
        pipeline.ops.basecurve.exposure = ev as f32;
    }

    if let Some(temp) = edits.white_balance {
        pipeline.ops.tolab.set_temp(temp as f32, 1.0);
    }
}
