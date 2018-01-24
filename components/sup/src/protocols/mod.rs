// Copyright (c) 2018 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod ctl;
pub mod types;

use std::path::PathBuf;
use std::str::FromStr;

use common::command::package::install::InstallSource;
use hcore::service::ApplicationEnvironment;

use manager::service::{IntoServiceSpec, ServiceBind, ServiceSpec};

impl IntoServiceSpec for ctl::SvcLoad {
    // Only use this for standalone services!
    fn into_spec(mut self, spec: &mut ServiceSpec) {
        spec.ident = InstallSource::from_str(self.get_source()).unwrap().into();
        spec.bldr_url = self.take_bldr_url();
        spec.channel = self.take_bldr_channel();
        if self.has_application_environment() {
            spec.application_environment =
                ApplicationEnvironment::from_str(self.get_application_environment()).ok();
        }
        spec.group = self.take_group();
        spec.update_strategy = self.get_update_strategy();
        spec.topology = self.get_topology();
        // spec.binds = self.get_binds()
        //     .iter()
        //     .map(ServiceBind::from_str)
        //     .collect()
        //     .into_vec();
        if self.has_config_from() {
            spec.config_from = Some(PathBuf::from(self.take_config_from()));
        }
        if self.has_svc_encrypted_password() {
            spec.svc_encrypted_password = Some(self.take_svc_encrypted_password());
        }
    }

    /// All specs in a composite currently share a lot of the same
    /// information. Here, we create a "base spec" that we can clone and
    /// further customize for each individual service as needed.
    fn into_composite_spec(mut self, spec: &mut ServiceSpec) {
        // All the composite's services are in the same composite,
        // tautologically enough!
        spec.composite = Some(
            InstallSource::from_str(self.get_source())
                .unwrap()
                .as_ref()
                .name
                .to_string(),
        );

        // All services will pull from the same channel in the same
        // Builder instance
        spec.bldr_url = self.take_bldr_url();
        spec.channel = self.take_bldr_channel();

        // All services will be in the same group and app/env. Binds among
        // the composite's services are generated based on this
        // assumption.
        //
        // (We do not set binds here, though, because that requires
        // specialized, service-specific handling.)
        if self.has_application_environment() {
            spec.application_environment =
                ApplicationEnvironment::from_str(self.get_application_environment()).ok();
        }
        spec.group = self.take_group();

        // For now, all a composite's services will also share the same
        // update strategy and topology, though we may want to revisit
        // this in the future (particularly for topology).
        spec.update_strategy = self.get_update_strategy();
        spec.topology = self.get_topology();

        // TODO (CM): Not dealing with service passwords for now, since
        // that's a Windows-only feature, and we don't currently build
        // Windows composites yet. And we don't have a nice way target
        // them on a per-service basis.

        // TODO (CM): Not setting the dev-mode service config_from value
        // because we don't currently have a nice way to target them on a
        // per-service basis.
    }

    // fn update_composite_service_specs(
    //     spec: &mut Vec<ServiceSpec>,
    //     package: &PackageInstall,
    //     m: &ArgMatches,
    // ) -> Result<()> {
    //     let bind_map = package.bind_map()?;
    //     // TODO (CM): maybe not mutable?
    //     let mut cli_composite_binds = composite_binds_from_input(m)?;
    //
    //     let update_binds = m.values_of("BIND").is_some();
    //
    //     for spec in spec.iter_mut() {
    //         // The Builder URL and channel have default values; we only want to
    //         // change them if the user specified something!
    //         set_bldr_url_from_input(spec, m);
    //         set_channel_from_input(spec, m);
    //
    //         set_app_env_from_input(spec, m)?;
    //         set_group_from_input(spec, m);
    //         set_strategy_from_input(spec, m);
    //         set_topology_from_input(spec, m);
    //
    //         // No setting of config or password either; see notes in
    //         // `base_composite_service_spec` for more.
    //
    //         // Just as with standalone services, we don't do anything to
    //         // the binds unless you've specified new ones on the CLI. For
    //         // composites, such binds can be thought of as binds for the
    //         // overall composite.
    //         if update_binds {
    //             set_composite_binds(spec, &bind_map, &mut cli_composite_binds)?;
    //         }
    //     }
    //     Ok(())
    // }
}
