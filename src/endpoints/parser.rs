use std::{fs::DirEntry, vec};

use log::warn;


#[derive(Debug)]
enum EndpointMethod { GET, POST }




#[derive(Debug)]
pub struct Endpoint {
    method: EndpointMethod,
    full_url_path: String,
    relative_url_path: String,
    file_path: String,
    file_content: String,

}

#[derive(Debug)]
pub struct Project {
    pub project_name: String,
    pub endpoints: Vec<Endpoint>,
}

impl Project {

    fn load_get_enpoints(e: &DirEntry) -> Box<dyn Iterator<Item = Endpoint>> {
        Box::new(std::iter::empty())
    }

    fn load_post_endpoints(e: &DirEntry) -> Box<dyn Iterator<Item = Endpoint>> {
        Box::new(std::iter::empty())
    }

    pub fn parse_from_dir_entry(entry: &DirEntry) -> Project {
        let name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
        let paths = std::fs::read_dir(entry.path()).unwrap();
 
        let iter = paths
            .map(|e| e.unwrap())
            .filter(|e| {
                if e.path().is_file() {
                    warn!("Skipping project {} has unexpected file {}", name, e.path().display());
                    false
                } else if ["GET", "POST"].contains(&e.file_name().to_str().unwrap()) {
                    true
                } else {
                    warn!("Skipping project {} unsupported method {}", name, e.file_name().to_str().unwrap());
                    false
                }
            })
            .map(|e| {
                if e.path().ends_with("GET") {
                    Project::load_get_enpoints(&e)
                } else if e.path().ends_with("POST") {
                    Project::load_post_endpoints(&e)
                } else {
                    panic!("something went wrong in parse_from_dir_entry")
                }
            })
            .reduce(|a, b| Box::new(a.chain(b))).unwrap();
        
        Project {
            project_name: name,
            endpoints: iter.collect(),
        }

        
    }
}

#[derive(Debug)]
pub struct EndpointCollections {
    pub projects: Vec<Project>,
}

impl EndpointCollections {
    pub fn parse_from_dir(dsl_dir: &String) -> EndpointCollections {
        let paths = std::fs::read_dir(&dsl_dir).unwrap();
        // let projects_count = paths.count();
        // let vec: Vec<Project> = Vec::with_capacity(projects_count);

        let projects = paths.into_iter()
            .map(|e| e.unwrap())
            .filter(|e| {
                let valid = e.path().is_dir();
                if !valid {
                    warn!("Skipping file {} because it's not a project", e.path().display());
                }
                valid
            })
            .map(
                |e| Project::parse_from_dir_entry(&e)
            )
            .collect();

        EndpointCollections { projects: projects }
    }
}

impl std::fmt::Display for EndpointCollections {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let projects_strings: Vec<String> = self.projects.iter().map(|p| format!("{}", p)).collect();

        write!(f, "{{ projects: [{}] }}", projects_strings.join(","))
    }
}


impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let endpoints_strings: Vec<String> = self.endpoints.iter().map(|e| format!("{}", e)).collect();
        write!(f, "{{ name: {}, endpoints: [{}] }}", self.project_name, endpoints_strings.join(","))
    }
}

impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ method: {:?}, url: {} }}", self.method, self.relative_url_path)
    }
}