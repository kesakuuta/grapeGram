package app.grapegram;

import android.content.Context;
import android.view.View;
import android.widget.FrameLayout;
import android.widget.Toast;

import org.telegram.messenger.AndroidUtilities;
import org.telegram.messenger.LocaleController;
import org.telegram.messenger.R;
import org.telegram.messenger.Utilities;
import org.telegram.ui.ActionBar.ActionBar;
import org.telegram.ui.ActionBar.AlertDialog;
import org.telegram.ui.ActionBar.BaseFragment;
import org.telegram.ui.ActionBar.Theme;
import org.telegram.ui.Components.IconBackgroundColors;
import org.telegram.ui.Components.LayoutHelper;
import org.telegram.ui.Components.UItem;
import org.telegram.ui.Components.UniversalAdapter;
import org.telegram.ui.Components.UniversalRecyclerView;
import org.telegram.ui.SettingsActivity.SettingCell;

import java.util.ArrayList;

import app.exteraless.ai.ui.AiSettingsActivity;
import app.exteraless.plugins.ui.PluginsActivity;
import app.exteraless.settings.OpenExteraAppearanceActivity;
import app.exteraless.settings.OpenExteraAyuMomentsActivity;
import app.exteraless.settings.OpenExteraChatsActivity;
import app.exteraless.settings.OpenExteraCloudActivity;
import app.exteraless.settings.OpenExteraGeneralActivity;
import app.exteraless.settings.OpenExteraOtherActivity;

/** The single grapeGram settings hub, built from Telegram's own settings cells. */
public class GrapeSettingsActivity extends BaseFragment {

    private static final int ID_APPEARANCE = 1;
    private static final int ID_CHATS = 2;
    private static final int ID_AYU = 3;
    private static final int ID_AI = 4;
    private static final int ID_PLUGINS = 5;
    private static final int ID_BADGES = 6;
    private static final int ID_CREDITS = 7;
    private static final int ID_GENERAL = 8;
    private static final int ID_OTHER = 9;
    private static final int ID_CLOUD = 10;

    private UniversalRecyclerView listView;

    @Override
    public View createView(Context context) {
        setHasOwnBackground(true);

        actionBar.setBackButtonImage(R.drawable.ic_ab_back);
        actionBar.setAllowOverlayTitle(true);
        actionBar.setTitle("grapeGram");
        actionBar.setActionBarMenuOnItemClick(new ActionBar.ActionBarMenuOnItemClick() {
            @Override
            public void onItemClick(int id) {
                if (id == -1) {
                    finishFragment();
                }
            }
        });

        FrameLayout contentView = new FrameLayout(context);
        contentView.setBackgroundColor(getThemedColor(Theme.key_windowBackgroundGray));

        listView = new UniversalRecyclerView(this, this::fillItems, this::onClick, null);
        listView.setSections();
        listView.adapter.setApplyBackground(false);
        listView.setSectionsDrawBackground(true);
        contentView.addView(listView, LayoutHelper.createFrame(LayoutHelper.MATCH_PARENT, LayoutHelper.MATCH_PARENT));
        actionBar.setAdaptiveBackground(listView);

        fragmentView = contentView;
        return fragmentView;
    }

    private void fillItems(ArrayList<UItem> items, UniversalAdapter adapter) {
        items.add(UItem.asHeader("grapeGram " + GrapeConfig.VERSION));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("Внешний вид"));
        items.add(row(ID_APPEARANCE, IconBackgroundColors.PURPLE, R.drawable.settings_account,
                "Оформление", "темы, анимации, Monet"));
        items.add(row(ID_CHATS, IconBackgroundColors.BLUE, R.drawable.settings_chat,
                "Чаты", "сообщения, папки, иконка приложения"));
        items.add(row(ID_AYU, IconBackgroundColors.ORANGE, R.drawable.settings_folders,
                "AyuMoments", "истории и статусы"));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("Поведение"));
        items.add(row(ID_GENERAL, IconBackgroundColors.GREEN_DEEP, R.drawable.msg_media,
                "Общие", "переводчик, сеть, уведомления"));
        items.add(row(ID_OTHER, IconBackgroundColors.GRAY, R.drawable.msg_fave,
                "Другое", "эксперименты и резервные копии"));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("Интеллект и расширения"));
        items.add(row(ID_AI, IconBackgroundColors.CYAN, R.drawable.msg_bot,
                "AI", "провайдеры и модели"));
        items.add(row(ID_PLUGINS, IconBackgroundColors.GREEN, R.drawable.settings_premium,
                "Плагины", "установка, разрешения, исходники, AI-анализ, журнал"));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("О приложении"));
        items.add(row(ID_BADGES, IconBackgroundColors.BLUE_DEEP, R.drawable.filled_profile_settings,
                "Бейджи", "подписанный список из GitHub"));
        items.add(row(ID_CLOUD, IconBackgroundColors.CYAN, R.drawable.cloud_sync,
                "Облако", "резервная копия настроек"));
        items.add(row(ID_CREDITS, IconBackgroundColors.ORANGE_DEEP, R.drawable.settings_ask,
                "Авторы и благодарности", "Cherrygram · exteraGram · exteraless"));
    }

    private static UItem row(int id, IconBackgroundColors colors, int icon, String title, String subtitle) {
        return SettingCell.Factory.of(id, colors.top, colors.bottom, icon, title, subtitle);
    }

    private void onClick(UItem item, View view, int position, float x, float y) {
        switch (item.id) {
            case ID_APPEARANCE:
                presentFragment(new OpenExteraAppearanceActivity());
                break;
            case ID_CHATS:
                presentFragment(new OpenExteraChatsActivity());
                break;
            case ID_AYU:
                presentFragment(new OpenExteraAyuMomentsActivity());
                break;
            case ID_GENERAL:
                presentFragment(new OpenExteraGeneralActivity());
                break;
            case ID_OTHER:
                presentFragment(new OpenExteraOtherActivity());
                break;
            case ID_AI:
                presentFragment(new AiSettingsActivity());
                break;
            case ID_PLUGINS:
                presentFragment(new PluginsActivity());
                break;
            case ID_BADGES:
                showBadges();
                break;
            case ID_CLOUD:
                presentFragment(new OpenExteraCloudActivity());
                break;
            case ID_CREDITS:
                presentFragment(new GrapeCreditsActivity());
                break;
            default:
                break;
        }
    }

    private void showBadges() {
        final Context context = getParentActivity();
        if (context == null) {
            return;
        }
        final AlertDialog progress = new AlertDialog(context, AlertDialog.ALERT_TYPE_SPINNER);
        progress.setCanCancel(true);
        progress.show();
        Utilities.globalQueue.postRunnable(() -> {
            final GrapeBadges.Result result = GrapeBadges.load(context.getApplicationContext(), true);
            AndroidUtilities.runOnUIThread(() -> {
                try {
                    progress.dismiss();
                } catch (Throwable ignored) {
                }
                if (!result.ok) {
                    toast("Бейджи: " + result.error);
                    return;
                }
                GrapeBadgeBridge.syncAsync(context.getApplicationContext());
                StringBuilder sb = new StringBuilder();
                for (GrapeBadges.Entry e : result.entries) {
                    if (e.enabled) {
                        sb.append(e.userId).append(" — ").append(e.badge).append('\n');
                    }
                }
                if (sb.length() == 0) {
                    sb.append("Пока никому не выданы");
                }
                new AlertDialog.Builder(context)
                        .setTitle("Бейджи · v" + result.version)
                        .setMessage(sb.toString().trim())
                        .setPositiveButton(LocaleController.getString(R.string.OK), null)
                        .show();
            });
        });
    }

    private void toast(String text) {
        if (getParentActivity() != null) {
            Toast.makeText(getParentActivity(), text, Toast.LENGTH_SHORT).show();
        }
    }
}
