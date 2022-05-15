mod label;
mod list_view;
use crate::WrappedWidget;
pub use list_view::*;

macro_rules! make_wrapped_widget {
	($name:ident, $ty:ty, $create:expr) => {
		pub fn $name() -> WrappedWidget<'static, $ty> {
			WrappedWidget::new($create)
		}
	};
}
macro_rules! make_simple {
	($ty:ident) => {
		paste::paste! {
			make_wrapped_widget!([<$ty:snake>], gtk::$ty, gtk::$ty::new());
		}
	};
}

make_wrapped_widget!(label, gtk::Label, gtk::Label::new(None));
make_simple!(Spinner);
make_simple!(Statusbar);
make_simple!(LevelBar);
make_simple!(ProgressBar);
make_simple!(InfoBar);
make_simple!(Image);
make_simple!(Picture);
make_wrapped_widget!(
	separator,
	gtk::Separator,
	gtk::Separator::new(gtk::Orientation::Vertical)
);
make_simple!(TextView);
make_wrapped_widget!(
	scale,
	gtk::Scale,
	gtk::Scale::new(gtk::Orientation::Vertical, Option::<&gtk::Adjustment>::None)
);
make_wrapped_widget!(
	window_controls,
	gtk::WindowControls,
	gtk::WindowControls::new(gtk::PackType::End)
);

make_simple!(Button);
make_simple!(ToggleButton);
make_wrapped_widget!(link_button, gtk::LinkButton, gtk::LinkButton::new(""));
make_simple!(CheckButton);
make_simple!(MenuButton);
make_simple!(Switch);
make_simple!(ColorButton);
make_simple!(FontButton);

make_simple!(Entry);
make_simple!(SearchEntry);
make_simple!(PasswordEntry);
make_wrapped_widget!(
	spin_button,
	gtk::SpinButton,
	gtk::SpinButton::new(Option::<&gtk::Adjustment>::None, 1.0, 0)
);
make_wrapped_widget!(
	editable_label,
	gtk::EditableLabel,
	gtk::EditableLabel::new("")
);

make_wrapped_widget!(
	container,
	gtk::Box,
	gtk::Box::new(gtk::Orientation::Vertical, 0)
);
make_simple!(Grid);
make_simple!(CenterBox);
make_simple!(ScrolledWindow);
make_wrapped_widget!(
	paned,
	gtk::Paned,
	gtk::Paned::new(gtk::Orientation::Vertical)
);
make_wrapped_widget!(frame, gtk::Frame, gtk::Frame::new(None));
make_wrapped_widget!(expander, gtk::Expander, gtk::Expander::new(None));
make_simple!(FlowBox);
make_simple!(Overlay);
make_simple!(Stack);
make_simple!(StackSwitcher);
make_simple!(StackSidebar);

make_simple!(Window);
make_simple!(Dialog);
make_simple!(AboutDialog);
