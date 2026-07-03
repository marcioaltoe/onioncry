use crate::*;
use std::collections::HashSet;
use std::path::Path;

pub(crate) struct FeatureSystemLocation {
    pub(crate) domain: String,
    pub(crate) system_path: String,
    pub(crate) relative_file: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FeatureSystemDependencyArea {
    PublicEntry,
    Adapters,
    Lib,
    Hooks,
    Contexts,
    Stores,
    Components,
    Guards,
    Routes,
    Other,
}

impl FeatureSystemDependencyArea {
    pub(crate) fn from_relative_file(relative_file: &str) -> Self {
        match relative_file.split('/').next().unwrap_or_default() {
            "adapters" => Self::Adapters,
            "lib" => Self::Lib,
            "hooks" => Self::Hooks,
            "contexts" => Self::Contexts,
            "stores" => Self::Stores,
            "components" => Self::Components,
            "guards" => Self::Guards,
            _ => Self::Other,
        }
    }

    pub(crate) fn config_name(self) -> &'static str {
        match self {
            Self::PublicEntry => "public-entry",
            Self::Adapters => "adapters",
            Self::Lib => "lib",
            Self::Hooks => "hooks",
            Self::Contexts => "contexts",
            Self::Stores => "stores",
            Self::Components => "components",
            Self::Guards => "guards",
            Self::Routes => "routes",
            Self::Other => "other",
        }
    }

    pub(crate) fn display_name(self) -> &'static str {
        match self {
            Self::PublicEntry => "public entry",
            Self::Adapters => "adapters",
            Self::Lib => "lib",
            Self::Hooks => "hooks",
            Self::Contexts => "contexts",
            Self::Stores => "stores",
            Self::Components => "components",
            Self::Guards => "guards",
            Self::Routes => "routes",
            Self::Other => "outside code",
        }
    }
}

impl FeatureSystemDependencyArea {
    pub(super) fn is_upper_frontend_area(self) -> bool {
        matches!(
            self,
            Self::Hooks | Self::Contexts | Self::Stores | Self::Components | Self::Guards
        )
    }
}

pub(super) fn system_location(
    project_root: &Path,
    file: &Path,
    systems_roots: &[Vec<String>],
) -> Option<FeatureSystemLocation> {
    let components = project_relative_components(project_root, file);
    systems_roots.iter().find_map(|root| {
        if components.len() <= root.len() || !path_has_prefix_components(&components, root) {
            return None;
        }
        let domain = components[root.len()].clone();
        let system_components = &components[..=root.len()];
        let relative_components = &components[root.len() + 1..];
        Some(FeatureSystemLocation {
            domain,
            system_path: display_path_components(system_components),
            relative_file: display_path_components(relative_components),
        })
    })
}

pub(super) fn is_public_entry(
    location: &FeatureSystemLocation,
    allowed_public_entry_points: &HashSet<String>,
) -> bool {
    allowed_public_entry_points.contains(&location.relative_file)
}

pub(super) fn is_route_file(project_root: &Path, file: &Path, route_roots: &[Vec<String>]) -> bool {
    let components = project_relative_components(project_root, file);
    path_under_any_root(&components, route_roots)
}

pub(super) fn is_file_in_area(location: &FeatureSystemLocation, area_directory: &str) -> bool {
    location
        .relative_file
        .split('/')
        .next()
        .is_some_and(|segment| segment == area_directory)
}
