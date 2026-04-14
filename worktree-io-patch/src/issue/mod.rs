mod deep_link;
mod def;
mod impls;
mod parse;
mod paths;
pub use deep_link::DeepLinkOptions;
pub use def::IssueRef;

#[cfg(test)]
mod adhoc_tests;
#[cfg(test)]
mod azure_paths_tests;
#[cfg(test)]
mod azure_tests;
#[cfg(test)]
mod gitlab_tests;
#[cfg(test)]
mod jira_tests;
#[cfg(test)]
mod linear_tests;
#[cfg(test)]
mod local_tests;
#[cfg(test)]
mod multi_dir_name_tests;
#[cfg(test)]
mod parse_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod uuid_tests;
