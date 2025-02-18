use crate::prelude::*;

/// Places content at an absolute position.
///
/// Placed content will not affect the position of other content. Place is
/// always relative to its parent container and will be in the foreground of all
/// other content in the container. Page margins will be respected.
///
///
/// ## Example { #example }
/// ```example
/// #set page(height: 60pt)
/// Hello, world!
///
/// #place(
///   top + right,
///   square(
///     width: 20pt,
///     stroke: 2pt + blue
///   ),
/// )
/// ```
///
/// Display: Place
/// Category: layout
#[element(Layout, Behave)]
pub struct PlaceElem {
    /// Relative to which position in the parent container to place the content.
    ///
    /// Cannot be `{auto}` if `float` is `{false}` and must be either
    /// `{auto}`, `{top}`, or `{bottom}` if `float` is `{true}`.
    ///
    /// When an axis of the page is `{auto}` sized, all alignments relative to
    /// that axis will be ignored, instead, the item will be placed in the
    /// origin of the axis.
    #[positional]
    #[default(Smart::Custom(Axes::with_x(Some(GenAlign::Start))))]
    pub alignment: Smart<Axes<Option<GenAlign>>>,

    /// Whether the placed element has floating layout.
    ///
    /// Floating elements are positioned at the top or bottom of the page,
    /// displacing in-flow content.
    ///
    /// ```example
    /// #set page(height: 150pt)
    /// #let note(where, body) = place(
    ///   center + where,
    ///   float: true,
    ///   clearance: 6pt,
    ///   rect(body),
    /// )
    ///
    /// #lorem(10)
    /// #note(bottom)[Bottom 1]
    /// #note(bottom)[Bottom 2]
    /// #lorem(40)
    /// #note(top)[Top]
    /// #lorem(10)
    /// ```
    pub float: bool,

    /// The amount of clearance the placed element has in a floating layout.
    #[default(Em::new(1.5).into())]
    #[resolve]
    pub clearance: Length,

    /// The horizontal displacement of the placed content.
    ///
    /// ```example
    /// #set page(height: 100pt)
    /// #for i in range(16) {
    ///   let amount = i * 4pt
    ///   place(center, dx: amount - 32pt, dy: amount)[A]
    /// }
    /// ```
    pub dx: Rel<Length>,

    /// The vertical displacement of the placed content.
    pub dy: Rel<Length>,

    /// The content to place.
    #[required]
    pub body: Content,
}

impl Layout for PlaceElem {
    #[tracing::instrument(name = "PlaceElem::layout", skip_all)]
    fn layout(
        &self,
        vt: &mut Vt,
        styles: StyleChain,
        regions: Regions,
    ) -> SourceResult<Fragment> {
        let mut frame = self.layout_inner(vt, styles, regions)?.into_frame();

        // If expansion is off, zero all sizes so that we don't take up any
        // space in our parent. Otherwise, respect the expand settings.
        let target = regions.expand.select(regions.size, Size::zero());
        frame.resize(target, Align::LEFT_TOP);

        Ok(Fragment::frame(frame))
    }
}

impl PlaceElem {
    /// Layout without zeroing the frame size.
    pub fn layout_inner(
        &self,
        vt: &mut Vt,
        styles: StyleChain,
        regions: Regions,
    ) -> SourceResult<Fragment> {
        // The pod is the base area of the region because for absolute
        // placement we don't really care about the already used area.
        let base = regions.base();
        let expand =
            Axes::new(base.x.is_finite(), base.y.is_finite() && !self.float(styles));

        let pod = Regions::one(base, expand);

        let float = self.float(styles);
        let alignment = self.alignment(styles);
        if float
            && !matches!(
                alignment,
                Smart::Auto
                    | Smart::Custom(Axes {
                        y: Some(GenAlign::Specific(Align::Top | Align::Bottom)),
                        ..
                    })
            )
        {
            bail!(self.span(), "floating placement must be `auto`, `top`, or `bottom`");
        } else if !float && alignment.is_auto() {
            return Err("automatic positioning is only available for floating placement")
                .hint("you can enable floating placement with `place(float: true, ..)`")
                .at(self.span());
        }

        let child = self
            .body()
            .moved(Axes::new(self.dx(styles), self.dy(styles)))
            .aligned(
                alignment.unwrap_or_else(|| Axes::with_x(Some(Align::Center.into()))),
            );

        child.layout(vt, styles, pod)
    }
}

impl Behave for PlaceElem {
    fn behaviour(&self) -> Behaviour {
        Behaviour::Ignorant
    }
}
