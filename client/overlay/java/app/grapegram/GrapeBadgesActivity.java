package app.grapegram;

import android.content.Context;
import android.graphics.Typeface;
import android.view.View;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;
import android.widget.Toast;

import org.telegram.messenger.AndroidUtilities;
import org.telegram.messenger.R;
import org.telegram.messenger.UserConfig;
import org.telegram.messenger.Utilities;
import org.telegram.ui.ActionBar.ActionBar;
import org.telegram.ui.ActionBar.BaseFragment;
import org.telegram.ui.ActionBar.Theme;

import java.util.List;

/** Full-page view of the currently signed grapeGram badge assignments. */
public final class GrapeBadgesActivity extends BaseFragment {

    private LinearLayout content;
    private TextView status;
    private LinearLayout entries;
    private Button refreshButton;

    @Override
    public View createView(Context context) {
        actionBar.setBackButtonImage(R.drawable.ic_ab_back);
        actionBar.setTitle("Бейджи grapeGram");
        actionBar.setActionBarMenuOnItemClick(new ActionBar.ActionBarMenuOnItemClick() {
            @Override
            public void onItemClick(int id) {
                if (id == -1) finishFragment();
            }
        });

        ScrollView scroll = new ScrollView(context);
        scroll.setFillViewport(true);
        scroll.setBackgroundColor(getThemedColor(Theme.key_windowBackgroundGray));

        content = new LinearLayout(context);
        content.setOrientation(LinearLayout.VERTICAL);
        content.setPadding(AndroidUtilities.dp(16), AndroidUtilities.dp(16),
                AndroidUtilities.dp(16), AndroidUtilities.dp(24));
        scroll.addView(content, new ScrollView.LayoutParams(
                ScrollView.LayoutParams.MATCH_PARENT, ScrollView.LayoutParams.WRAP_CONTENT));

        TextView description = new TextView(context);
        description.setText("Здесь отображаются назначения, полученные из общего источника и проверенные подписью Ed25519. Последний проверенный список остаётся доступен без сети.");
        description.setTextSize(14);
        description.setTextColor(getThemedColor(Theme.key_windowBackgroundWhiteBlackText));
        description.setPadding(AndroidUtilities.dp(4), AndroidUtilities.dp(4),
                AndroidUtilities.dp(4), AndroidUtilities.dp(14));
        content.addView(description, new LinearLayout.LayoutParams(-1, -2));

        status = new TextView(context);
        status.setTextSize(13);
        status.setTextColor(getThemedColor(Theme.key_windowBackgroundWhiteGrayText));
        status.setPadding(AndroidUtilities.dp(4), AndroidUtilities.dp(6),
                AndroidUtilities.dp(4), AndroidUtilities.dp(10));
        content.addView(status, new LinearLayout.LayoutParams(-1, -2));

        entries = new LinearLayout(context);
        entries.setOrientation(LinearLayout.VERTICAL);
        content.addView(entries, new LinearLayout.LayoutParams(-1, -2));

        refreshButton = addButton(context, "Обновить подписанный список", this::refresh);
        long ownerId = UserConfig.getInstance(UserConfig.selectedAccount).clientUserId;
        if (GrapeConfig.isBadgeOwner(ownerId)) {
            addButton(context, "Управление бейджами", () ->
                    presentFragment(new GrapeBadgeAdminActivity()));
        }

        fragmentView = scroll;
        refresh();
        return fragmentView;
    }

    private Button addButton(Context context, String title, Runnable action) {
        Button button = new Button(context);
        button.setText(title);
        button.setAllCaps(false);
        button.setOnClickListener(v -> action.run());
        LinearLayout.LayoutParams lp = new LinearLayout.LayoutParams(-1, -2);
        lp.topMargin = AndroidUtilities.dp(8);
        content.addView(button, lp);
        return button;
    }

    private void refresh() {
        if (status == null || getParentActivity() == null) return;
        refreshButton.setEnabled(false);
        status.setText("Проверка подписи и загрузка…");
        entries.removeAllViews();
        Context appContext = getParentActivity().getApplicationContext();
        Utilities.globalQueue.postRunnable(() -> {
            GrapeBadges.Result result = GrapeBadges.load(appContext, true);
            AndroidUtilities.runOnUIThread(() -> {
                if (status == null || getParentActivity() == null) return;
                refreshButton.setEnabled(true);
                if (!result.ok) {
                    status.setText("Не удалось получить новый список: " + result.error
                            + "\nЕсли ранее был сохранён проверенный список, он продолжит использоваться для отображения бейджей.");
                    return;
                }
                GrapeBadgeBridge.sync(appContext, result);
                status.setText("Подпись действительна · версия " + result.version
                        + " · записей: " + result.entries.size());
                renderEntries(result.entries);
            });
        });
    }

    private void renderEntries(List<GrapeBadges.Entry> list) {
        entries.removeAllViews();
        boolean any = false;
        for (GrapeBadges.Entry entry : list) {
            if (entry == null || !entry.enabled) continue;
            any = true;
            TextView row = new TextView(getParentActivity());
            row.setText(entry.userId + "  ·  " + entry.badge);
            row.setTextSize(15);
            row.setTypeface(Typeface.DEFAULT, Typeface.BOLD);
            row.setTextColor(getThemedColor(Theme.key_windowBackgroundWhiteBlackText));
            row.setBackground(Theme.createRoundRectDrawable(AndroidUtilities.dp(12),
                    getThemedColor(Theme.key_windowBackgroundWhite)));
            row.setPadding(AndroidUtilities.dp(14), AndroidUtilities.dp(12),
                    AndroidUtilities.dp(14), AndroidUtilities.dp(12));
            LinearLayout.LayoutParams lp = new LinearLayout.LayoutParams(-1, -2);
            lp.topMargin = AndroidUtilities.dp(6);
            entries.addView(row, lp);
        }
        if (!any) {
            TextView empty = new TextView(getParentActivity());
            empty.setText("Активных бейджей пока нет.");
            empty.setTextSize(14);
            empty.setTextColor(getThemedColor(Theme.key_windowBackgroundWhiteGrayText));
            empty.setPadding(AndroidUtilities.dp(4), AndroidUtilities.dp(12),
                    AndroidUtilities.dp(4), AndroidUtilities.dp(12));
            entries.addView(empty, new LinearLayout.LayoutParams(-1, -2));
        }
    }

    @Override
    public void onFragmentDestroy() {
        status = null;
        entries = null;
        refreshButton = null;
        content = null;
        super.onFragmentDestroy();
    }
}
