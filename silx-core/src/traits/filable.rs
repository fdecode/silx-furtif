use std::{ path::{ Path, PathBuf, }, io::BufReader, fs::{ DirBuilder, File, }, mem, };
use serde::{ Serialize, Deserialize, de::DeserializeOwned, };

/// Properties of a type containing a variable data structure partially stored by means of several serialization files
pub trait Filable { 
    /// Type of the unfiled data type
    type Unfiled;

    /// Unload all data (filed data needs to be fully loaded) into the associated unfiled data type
    /// * Output: unfiled data or error
    fn unwrap(mut self) -> Result<Self::Unfiled,String,> where Self: Sized { self.unload(None) }

    /// Load all missing data from files; bool indicates if already loaded (false) or not
    /// * `dir: P` : directory path from which is loaded the serialized files of missing data
    /// * `P: AsRef<Path>` : type of the path
    /// * Output: a boolean or an error
    fn load<P: AsRef<Path>>(&mut self, dir: P) -> Result<bool,String>;

    /// Optionally serialize all data on disk (filed data needs to be fully loaded) while unloading into the associated unfiled data type
    /// * `odir: Option<&Path>` : optional directory path where are saved the serialized files of all data; if `None`, nothing is saved
    /// * Output: unfiled data or error
    fn unload(&mut self, odir: Option<&Path>) -> Result<Self::Unfiled,String,>;
}

#[derive(Clone, Serialize, Deserialize, Debug,)]
/// Filed data type
/// * `T` : unfiled data type associated to the filed data type
pub enum Filed<T> {
    #[allow(non_camel_case_types)]
    /// Unloaded variant 
    unloaded {
        /// File path for serialized data
        path: PathBuf,
    },
    #[allow(non_camel_case_types)]
    /// Loaded variant
    loaded {
        /// File path for serialized data
        path: PathBuf,
        /// Unfiled data
        data: T
    },
}

#[derive(Clone, Serialize, Deserialize, Debug,)]
/// Recursive filed data type
/// * `T: Filable` : Partially unfiled data type associated to the filed data type
///   * This partially unfiled data type is not an unfiled data type
///   * Indeed, the unloading process is recursive for this structure
pub enum RecFiled<T: Filable> {
    #[allow(non_camel_case_types)]
    /// Unloaded variant 
    unloaded {
        /// File path for serialized data
        path: PathBuf,
    },
    #[allow(non_camel_case_types)]
    /// Partially loaded variant
    partially_loaded {
        /// File path for serialized data
        path: PathBuf,
        /// Partially unfiled data
        data: T
    },
}
impl<T> Filed<T> {
    /// Builder for unloaded variant
    /// * `path: P` : path of serialization file
    /// * `P: AsRef<Path>` : type of the path 
    /// * Output: the variant
    pub fn new_unloaded<P: AsRef<Path>>(path: P,) -> Self { 
        let path = path.as_ref().to_path_buf();
        Self::unloaded { path, } 
    }

    /// Builder for loaded variant
    /// * `path: P` : path of serialization file
    /// * `data: T` : unfiled data
    /// * `P: AsRef<Path>` : type of the path 
    /// * Output: the variant
    pub fn new_loaded<P: AsRef<Path>>(path: P, data: T) -> Self { 
        let path = path.as_ref().to_path_buf();
        Self::loaded { path, data, } 
    }
}

impl<T: Filable> RecFiled<T> {
    /// Builder for unloaded variant
    /// * `path: P` : path of serialization file
    /// * `P: AsRef<Path>` : type of the path 
    /// * Output: the variant
    pub fn new_unloaded<P: AsRef<Path>>(path: P,) -> Self { 
        let path = path.as_ref().to_path_buf();
        Self::unloaded { path, } 
    }

    /// Builder for partially loaded variant
    /// * `path: P` : path of serialization file
    /// * `data: T` : partially unfiled data
    /// * `P: AsRef<Path>` : type of the path 
    /// * Output: the variant
    pub fn new_partially_loaded<P: AsRef<Path>>(path: P, data: T) -> Self { 
        let path = path.as_ref().to_path_buf();
        Self::partially_loaded { path, data, } 
    }
}

impl<T> Filable for Filed<T> where T: Serialize + DeserializeOwned, {
    type Unfiled = T;

    fn load<P: AsRef<Path>,>(&mut self, prefix: P,) -> Result<bool,String> {
        let opath = match self { 
            Filed::loaded{..} => None, 
            Filed::unloaded { path } => Some(path.clone()), 
        };
        match opath {
            None => { Ok(false) },
            Some(path) => { 
                let full_path = prefix.as_ref().join(&path);
                let reader = File::open(&full_path).map(|f| BufReader::new(f));
                if let Ok(reader) = reader { 
                    if let Ok(data) = serde_yaml::from_reader::<_,T>(reader) {
                        *self = Filed::loaded { path, data }; 
                        Ok(true)
                    } else { Err("load: failed to unserialize".to_string()) }
                } else { Err("load: failed to open file".to_string()) }
            }
        }
    }

    // optionally save all data on disk (all data should be loaded) while unloading data structure, and return unfiled data
    // opath indicate directory from which to load; if none, then data is actually not saved!
    // proc is the pre-process run before unloading
    fn unload(&mut self, opath: Option<&Path>,) -> Result<T,String,> {
        match if let Filed::loaded { data, path, } = self { 
            let opath_prefix = opath.map(|prefix| {                                                     // | create path if necessary
                if let Some(path) = path.parent() { prefix.join(path) } else { prefix.to_path_buf() }   // |
            });                                                                                         // |
            let mut err = Ok(());                                                                       // |
            if let Some(path) = opath_prefix { err = DirBuilder::new().recursive(true).create(path); }  // |
            if err.is_err() { Err("save: failed to build path".to_string()) } else {
                let filed = Filed::unloaded { path: path.to_path_buf(), };
                let opath = opath.map(|prefix| prefix.join(path));
                if let Some(path) = opath {
                    let buffer = File::create(path);
                    if let Ok(buffer) = buffer {
                        match serde_yaml::to_writer(buffer,data) {
                            Ok(_) => Ok(filed),
                            Err(e) => Err(format!(
                                "save_unload: failed to serialize -> {e}"
                            )),
                        }
                    } else { Err("save_unload: failed to open file".to_string()) }
                } else { Ok(filed) }
            }
        } else { Err("save_unload: unloaded data".to_string()) } { // matched data computed here
            Ok(mut filed) => { 
                mem::swap(&mut filed, self); 
                if let Filed::loaded { data, .. } = filed { Ok(data) } else { Err("unload: unexpected error".to_string()) }
            },
            Err(e)     => Err(e),
        }
    }
} 


impl<T> Filable for RecFiled<T> where T: Filable + Serialize + DeserializeOwned, {
    type Unfiled = T::Unfiled;

    fn load<P: AsRef<Path>,>(&mut self, prefix: P,) -> Result<bool,String> {
        let opath = match self { 
            RecFiled::partially_loaded{..} => None, 
            RecFiled::unloaded { path } => Some(path.clone()), 
        };
        match opath {
            None => { Ok(false) },
            Some(path) => { 
                let full_path = prefix.as_ref().join(&path);
                let reader = File::open(&full_path).map(|f| BufReader::new(f));
                if let Ok(reader) = reader { 
                    if let Ok(mut data) = serde_yaml::from_reader::<_,T>(reader) {
                        data.load(prefix)?;
                        *self = RecFiled::partially_loaded { path, data }; 
                        Ok(true)
                    } else { Err("load: failed to unserialize".to_string()) }
                } else { Err("load: failed to open file".to_string()) }
            }
        }
    }

    fn unload(&mut self, opath: Option<&Path>,) -> Result<Self::Unfiled,String,> {
        match if let RecFiled::partially_loaded { data, path, } = self { 
            let inner_data = data.unload(opath.clone())?;
            let opath_prefix = opath.map(|prefix| {                                                     // | create path if necessary
                if let Some(path) = path.parent() { prefix.join(path) } else { prefix.to_path_buf() }   // |
            });                                                                                         // |
            let mut err = Ok(());                                                                       // |
            if let Some(path) = opath_prefix { err = DirBuilder::new().recursive(true).create(path); }  // |
            if err.is_err() { Err("save: failed to build path".to_string()) } else {
                let filed = RecFiled::unloaded { path: path.to_path_buf(), };
                let opath = opath.map(|prefix| prefix.join(path));
                if let Some(path) = opath {
                    let buffer = File::create(path);
                    if let Ok(buffer) = buffer {
                        if serde_yaml::to_writer(buffer,data).is_ok() { Ok((inner_data,filed)) } 
                        else {  Err("save_unload: failed to serialize".to_string()) }
                    } else { Err("save_unload: failed to open file".to_string()) }
                } else { Ok((inner_data,filed)) }
            }
        } else { Err("save_unload: unloaded data".to_string()) } { // matched data computed here
            Ok((inner_data,mut filed)) => { 
                mem::swap(&mut filed, self); Ok(inner_data)
            },
            Err(e)     => Err(e),
        }
    }
}