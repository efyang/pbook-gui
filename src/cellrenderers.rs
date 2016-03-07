use gtk;
use gtk::{IsA, CellRenderer};
use gtkdef::SetEllipsizeMode;
use pango_sys::PangoEllipsizeMode;

pub enum AddMode {
    PackEnd,
    PackStart,
}

pub trait AddCellRenderers {
    fn add_cell_renderer_column<T: IsA<CellRenderer>>(&self,
                                                      title: &str,
                                                      cell: &T,
                                                      fill: bool,
                                                      resizable: bool,
                                                      expand: bool,
                                                      add_mode: AddMode,
                                                      attribute_type: &str,
                                                      column_number: i32);
    fn add_text_renderer_column(&self,
                                title: &str,
                                fill: bool,
                                resizable: bool,
                                expand: bool,
                                add_mode: AddMode,
                                column_number: i32);
    fn add_progress_renderer_column(&self,
                                    title: &str,
                                    fill: bool,
                                    resizable: bool,
                                    expand: bool,
                                    add_mode: AddMode,
                                    column_number: i32);
    fn add_toggle_renderer_column(&self,
                                  title: &str,
                                  fill: bool,
                                  resizable: bool,
                                  expand: bool,
                                  add_mode: AddMode,
                                  column_number: i32)
                                  -> gtk::CellRendererToggle;
}

impl AddCellRenderers for gtk::TreeView {
    fn add_cell_renderer_column<T: IsA<CellRenderer>>(&self,
                                                      title: &str,
                                                      cell: &T,
                                                      fill: bool,
                                                      resizable: bool,
                                                      expand: bool,
                                                      add_mode: AddMode,
                                                      attribute_type: &str,
                                                      column_number: i32) {
        let column = gtk::TreeViewColumn::new();
        match add_mode {
            AddMode::PackEnd => {
                column.pack_end(cell, fill);
            }
            AddMode::PackStart => {
                column.pack_start(cell, fill);
            }
        }
        column.add_attribute(cell, attribute_type, column_number);
        column.set_title(title);
        column.set_resizable(resizable);
        column.set_expand(expand);
        self.append_column(&column);
    }

    fn add_text_renderer_column(&self,
                                title: &str,
                                fill: bool,
                                resizable: bool,
                                expand: bool,
                                add_mode: AddMode,
                                column_number: i32) {
        let cell = gtk::CellRendererText::new();
        cell.set_ellipsize_mode(PangoEllipsizeMode::End);
        self.add_cell_renderer_column(title,
                                      &cell,
                                      fill,
                                      resizable,
                                      expand,
                                      add_mode,
                                      "text",
                                      column_number);
    }

    fn add_progress_renderer_column(&self,
                                    title: &str,
                                    fill: bool,
                                    resizable: bool,
                                    expand: bool,
                                    add_mode: AddMode,
                                    column_number: i32) {
        let cell = gtk::CellRendererProgress::new();
        self.add_cell_renderer_column(title,
                                      &cell,
                                      fill,
                                      resizable,
                                      expand,
                                      add_mode,
                                      "value",
                                      column_number);
    }

    fn add_toggle_renderer_column(&self,
                                  title: &str,
                                  fill: bool,
                                  resizable: bool,
                                  expand: bool,
                                  add_mode: AddMode,
                                  column_number: i32)
                                  -> gtk::CellRendererToggle {
        let cell = gtk::CellRendererToggle::new();
        cell.set_activatable(true);
        self.add_cell_renderer_column(title,
                                      &cell,
                                      fill,
                                      resizable,
                                      expand,
                                      add_mode,
                                      "active",
                                      column_number);
        cell
    }
}
