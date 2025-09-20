use gstreamer::prelude::*;
use tracing::{error, info, warn};

pub struct PipelineService {
    pipeline: gstreamer::Pipeline,
}

impl PipelineService {
    pub fn new(pipeline_string: &str) -> anyhow::Result<Self> {
        let pipeline = gstreamer::parse_launch(pipeline_string)?
            .downcast::<gstreamer::Pipeline>()
            .map_err(|_| anyhow::anyhow!("Failed to create pipeline"))?;

        Ok(PipelineService { pipeline })
    }

    pub fn start(&self) -> anyhow::Result<()> {
        info!("Starting pipeline");
        self.pipeline.set_state(gstreamer::State::Playing)?;
        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        info!("Stopping pipeline");
        self.pipeline.set_state(gstreamer::State::Null)?;
        Ok(())
    }

    pub fn pause(&self) -> anyhow::Result<()> {
        info!("Pausing pipeline");
        self.pipeline.set_state(gstreamer::State::Paused)?;
        Ok(())
    }

    pub fn get_state(&self) -> gstreamer::State {
        self.pipeline.current_state()
    }

    pub fn wait_for_completion(&self) -> anyhow::Result<()> {
        let bus = self.pipeline.bus().expect("Pipeline without bus");

        for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
            use gstreamer::MessageView;

            match msg.view() {
                MessageView::Eos(..) => {
                    info!("Pipeline completed successfully");
                    break;
                }
                MessageView::Error(err) => {
                    error!(
                        "Pipeline error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    return Err(anyhow::anyhow!("Pipeline error: {}", err.error()));
                }
                MessageView::Warning(warning) => {
                    warn!(
                        "Pipeline warning from {:?}: {} ({:?})",
                        warning.src().map(|s| s.path_string()),
                        warning.error(),
                        warning.debug()
                    );
                }
                MessageView::StateChanged(state_changed) => {
                    if state_changed
                        .src()
                        .map(|s| s == &self.pipeline)
                        .unwrap_or(false)
                    {
                        info!(
                            "Pipeline state changed from {:?} to {:?}",
                            state_changed.old(),
                            state_changed.current()
                        );
                    }
                }
                _ => (),
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct MediaInfo {
    pub duration: Option<u64>, // in seconds
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bitrate: Option<u32>,
    pub format: String,
}
