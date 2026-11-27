package app.grapegram;

import android.graphics.Canvas;
import android.graphics.ColorFilter;
import android.graphics.Paint;
import android.graphics.PixelFormat;
import android.graphics.Rect;
import android.graphics.drawable.Drawable;

/** Compact, theme-independent grapeGram badge that can sit beside a Telegram status drawable. */
public final class GrapeBadgeDrawable extends Drawable implements Drawable.Callback {

    private final Drawable primary;
    private final Drawable icon;
    private final Paint background = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final int size;
    private final int gap;
    private final int badgeColor;
    private int alpha = 255;

    public GrapeBadgeDrawable(Drawable primary, Drawable icon, int badgeColor, int size, int gap) {
        this.primary = primary;
        this.icon = icon;
        this.badgeColor = badgeColor;
        this.size = Math.max(1, size);
        this.gap = Math.max(0, gap);
        if (primary != null) {
            primary.setCallback(this);
        }
        if (icon != null) {
            icon.setCallback(this);
        }
        background.setColor(badgeColor);
    }

    public static int colorForBadge(String badge) {
        if ("founder".equals(badge)) {
            return 0xFFE0A52B;
        }
        if ("developer".equals(badge)) {
            return 0xFF8757D9;
        }
        if ("contributor".equals(badge)) {
            return 0xFF259F91;
        }
        return 0xFF7852C8;
    }

    @Override
    public void draw(Canvas canvas) {
        Rect bounds = getBounds();
        int left = bounds.left;
        int top = bounds.top;
        if (primary != null) {
            primary.setBounds(left, top, left + size, top + size);
            primary.setAlpha(alpha);
            primary.draw(canvas);
            left += size + gap;
        }
        float cx = left + size / 2f;
        float cy = top + size / 2f;
        background.setAlpha(alpha);
        canvas.drawCircle(cx, cy, size / 2f, background);
        if (icon != null) {
            int inset = Math.max(1, size / 7);
            icon.setBounds(left + inset, top + inset, left + size - inset, top + size - inset);
            icon.setAlpha(alpha);
            icon.draw(canvas);
        }
    }

    @Override
    public int getIntrinsicWidth() {
        return size + (primary == null ? 0 : size + gap);
    }

    @Override
    public int getIntrinsicHeight() {
        return size;
    }

    @Override
    public void setAlpha(int value) {
        alpha = value;
        invalidateSelf();
    }

    @Override
    public void setColorFilter(ColorFilter filter) {
        if (icon != null) {
            icon.setColorFilter(filter);
        }
        invalidateSelf();
    }

    @Override
    public int getOpacity() {
        return PixelFormat.TRANSLUCENT;
    }

    @Override
    public void invalidateDrawable(Drawable who) {
        invalidateSelf();
    }

    @Override
    public void scheduleDrawable(Drawable who, Runnable what, long when) {
        scheduleSelf(what, when);
    }

    @Override
    public void unscheduleDrawable(Drawable who, Runnable what) {
        unscheduleSelf(what);
    }
}
